import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:hotkey_manager/hotkey_manager.dart';
import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:logging/logging.dart';
import 'package:window_manager/window_manager.dart';

import 'app.dart';
import 'l10n/app_localizations.dart';
import 'platform/cursor_locator.dart' show findDisplayForPoint;
import 'platform/hotkey_service.dart';
import 'platform/local_file_url.dart';
import 'platform/macos/mac_window_channel.dart';
import 'platform/platform_bindings.dart';
import 'platform/tray_controller.dart';
import 'platform/windows/win_package_context.dart';
import 'providers.dart';
import 'ui/picker/picker_layout.dart';

final _log = Logger('Bootstrap');

Future<void> bootstrap(PlatformBindings bindings, List<String> args) async {
  initLogging(bindings.logFile);

  _log.info('LinkUnbound starting (msix=${isRunningInMsix()})');

  try {
    if (await bindings.tryDelegate(bindings.initialEvent)) {
      exit(0);
    }
  } on Object catch (e, st) {
    _log.warning('Delegation check failed', e, st);
  }

  bool claimed;
  try {
    claimed = await bindings.claim();
  } on Object catch (e, st) {
    _log.severe('claim() crashed', e, st);
    claimed = false;
  }

  if (!claimed) {
    // claim() returned false means the mutex was held; the resident's pipe is
    // now guaranteed to be listening (claim waits for readiness). Retry once.
    try {
      if (await bindings.tryDelegate(bindings.initialEvent)) {
        exit(0);
      }
    } on Object catch (e, st) {
      _log.warning('Post-claim delegation retry failed', e, st);
    }
    // Delegation failed again: the resident may have exited in between. Make
    // one last attempt to become the resident before dropping the event.
    try {
      claimed = await bindings.claim();
    } on Object catch (e, st) {
      _log.severe('Second claim() attempt crashed', e, st);
      claimed = false;
    }
    if (!claimed) {
      // Last-resort delegation before giving up: the resident that raced us may
      // now be ready to receive the pipe message.
      try {
        if (await bindings.tryDelegate(bindings.initialEvent)) {
          exit(0);
        }
      } on Object catch (e, st) {
        _log.warning('Final delegation attempt failed', e, st);
      }
      final eventType = bindings.initialEvent?.runtimeType;
      if (eventType != null) {
        _log.severe(
          'Discarding initial event: no resident could be reached '
          '(type=$eventType)',
        );
      }
      exit(0);
    }
  }

  final browserService = BrowserService(
    configFile: bindings.browsersFile,
    browserDetector: bindings.browserDetector,
  );
  final ruleService = RuleService(rulesFile: bindings.rulesFile);

  final isFirstBoot = !bindings.browsersFile.existsSync();

  try {
    await browserService.load();
  } on Object catch (e, st) {
    _log.severe('Browser config corrupted, resetting', e, st);
    try {
      await browserService.reset();
    } on Object catch (e, st) {
      _log.warning('Browser reset failed', e, st);
    }
  }

  if (isFirstBoot) {
    try {
      await _firstBootEarlyPhase(
        browserService: browserService,
        iconsDir: bindings.iconsDir,
      );
    } on Object catch (e, st) {
      _log.severe('First boot early phase failed (non-fatal)', e, st);
    }
  }

  try {
    await ruleService.load();
  } on Object catch (e, st) {
    _log.severe('Rules config corrupted, ignoring', e, st);
  }

  try {
    await windowManager.ensureInitialized();
    await windowManager.setPreventClose(true);
    await windowManager.waitUntilReadyToShow(
      const WindowOptions(
        titleBarStyle: TitleBarStyle.hidden,
        size: Size(640, 700),
        center: false,
        // Force a fully opaque background so compositors that lack Mica /
        // DWM acrylic (Windows 10 integrated GPUs, Remote Desktop) don't try
        // to render a transparent frame and crash the Flutter engine.
        backgroundColor: Color(0xFF1E1E1E),
      ),
      () async {
        await windowManager.setSkipTaskbar(true);
        if (!Platform.isMacOS) {
          try {
            await windowManager.setHasShadow(false);
          } on Object catch (e) {
            _log.fine('setHasShadow not supported: $e');
          }
          await windowManager.setPosition(const Offset(-9999, -9999));
          await windowManager.hide();
        }
      },
    );
  } on Object catch (e, st) {
    _log.severe('Window manager init failed', e, st);
  }

  // Created here so that the exitApp callback can call hotkeyService.dispose()
  // without needing to update the override after the container is built.
  final hotkeyService = HotkeyService();

  final container = ProviderContainer(
    overrides: [
      browserServiceProvider.overrideWithValue(browserService),
      ruleServiceProvider.overrideWithValue(ruleService),
      registrationServiceProvider.overrideWithValue(
        bindings.registrationService,
      ),
      startupServiceProvider.overrideWithValue(bindings.startupService),
      iconExtractorProvider.overrideWithValue(bindings.iconExtractor),
      iconsDirProvider.overrideWithValue(bindings.iconsDir),
      launchServiceProvider.overrideWithValue(bindings.launchService),
      localeFileProvider.overrideWithValue(bindings.localeFile),
      edgeWarningFileProvider.overrideWithValue(bindings.edgeWarningFile),
      hideTrayFileProvider.overrideWithValue(bindings.hideTrayFile),
      globalHotkeyFileProvider.overrideWithValue(bindings.globalHotkeyFile),
      appDataDirProvider.overrideWithValue(bindings.appDataDir),
      exitAppProvider.overrideWithValue(() async {
        try {
          await hotkeyService.dispose();
        } on Object catch (e, st) {
          _log.warning('Hotkey dispose failed during exit', e, st);
        }
        try {
          await bindings.release();
        } on Object catch (e, st) {
          _log.warning('Release failed during exit', e, st);
        }
        exit(0);
      }),
    ],
  );

  final macWindow = Platform.isMacOS ? MacWindowChannel() : null;

  container.listen<AppState>(appStateProvider, (prev, next) async {
    try {
      await _applyAppMode(prev, next, container, bindings, macWindow);
    } on Object catch (e, st) {
      _log.warning('App mode transition failed', e, st);
    }
  });

  // Subscribe to inbound events before runApp so no event is dropped while
  // Flutter initialises. The ready signal fires only when this listener
  // attaches (see MacInboundEvents._signalReadyOnce).
  bindings.inboundEvents.listen(
    (event) {
      try {
        switch (event) {
          case OpenUrlEvent(:final url):
            _handleUrl(url, container);
          case ShowSettingsEvent():
            container.read(appStateProvider.notifier).showSettings();
        }
      } on Object catch (e, st) {
        _log.warning('Inbound event handler failed', e, st);
      }
    },
    onError: (Object e, StackTrace st) {
      _log.warning('Inbound event stream error', e, st);
    },
  );

  runApp(
    UncontrolledProviderScope(container: container, child: const NavigateApp()),
  );

  WidgetsBinding.instance.addPostFrameCallback((_) async {
    if (bindings.startsHidden) return;
    if (container.read(appStateProvider).mode != AppMode.hidden) return;
    container.read(appStateProvider.notifier).showSettings();
  });

  // Tray init, icon extraction and update check are off the critical path:
  // defer them so the first frame renders (and the ready signal fires) before
  // doing IO or network.
  WidgetsBinding.instance.addPostFrameCallback((_) async {
    final hideTray = container.read(hideTrayProvider);
    if (!hideTray) {
      try {
        await _initTray(bindings, container);
      } on Object catch (e, st) {
        _log.severe('Tray init failed (non-fatal)', e, st);
      }
    }

    hotkeyService.setCallback(
      () => container.read(appStateProvider.notifier).showSettings(),
    );

    final savedHotkey = container.read(globalHotkeyProvider);
    try {
      await hotKeyManager.unregisterAll();
      await hotkeyService.register(savedHotkey);
    } on Object catch (e, st) {
      _log.warning('Hotkey registration failed (non-fatal)', e, st);
    }

    // Re-register hotkey when the setting changes and manage tray visibility.
    container.listen<String?>(globalHotkeyProvider, (prev, next) async {
      try {
        await hotkeyService.register(next);
      } on Object catch (e, st) {
        _log.warning('Hotkey re-registration failed', e, st);
      }
    });

    container.listen<bool>(hideTrayProvider, (prev, next) async {
      try {
        if (next) {
          await bindings.trayController.dispose();
        } else {
          await _initTray(bindings, container);
        }
      } on Object catch (e, st) {
        _log.warning('Tray visibility toggle failed', e, st);
      }
    });

    if (isFirstBoot) {
      try {
        await _firstBootLatePhase(
          browserService: browserService,
          iconExtractor: bindings.iconExtractor,
          iconsDir: bindings.iconsDir,
          registrationService: bindings.registrationService,
          executablePath: bindings.executablePath,
          skipRegistration: isRunningInMsix(),
          container: container,
        );
      } on Object catch (e, st) {
        _log.severe('First boot late phase failed (non-fatal)', e, st);
      }
    }

    // The update check is driven lazily by the first ref.watch(updateInfoProvider)
    // in the UI (picker footer). No explicit pre-fetch needed here.
  });
}

Future<void> _applyAppMode(
  AppState? prev,
  AppState next,
  ProviderContainer container,
  PlatformBindings bindings,
  MacWindowChannel? macWindow,
) async {
  if (prev?.mode == next.mode) {
    if (next.mode == AppMode.settings) {
      await windowManager.show();
      await windowManager.focus();
      await macWindow?.activate();
    }
    return;
  }
  switch (next.mode) {
    case AppMode.hidden:
      await windowManager.hide();
      // Settings mode cleared skipTaskbar; restore it so the hidden window
      // never lingers in the Windows taskbar.
      await windowManager.setSkipTaskbar(true);
      await macWindow?.setAccessory();
    case AppMode.settings:
      await macWindow?.setRegular();
      await macWindow?.setSettingsMode();
      // On macOS the settings window has a native title bar (~28px) that eats
      // into the frame; compensate so the content keeps its designed height.
      await windowManager.setSize(
        Size(640, Platform.isMacOS ? 728 : 700),
      );
      await windowManager.center();
      await windowManager.setSkipTaskbar(false);
      await windowManager.setAlwaysOnTop(false);
      await windowManager.show();
      await windowManager.focus();
      await macWindow?.activate();
    case AppMode.picker:
      try {
        await macWindow?.setPickerMode();
        final browsers = container.read(browsersProvider);
        final winSize = PickerLayout.windowSize(browsers.length);
        // Fetch cursor and display list concurrently, then hit-test locally so
        // both reads observe the same cursor position.
        final (cursorResult, rects) = await (
          bindings.cursorLocator.cursorPosition(),
          bindings.cursorLocator.displayRects(),
        ).wait;
        final (cursorX, cursorY) = cursorResult;
        final (originX, originY, displayW, displayH) = findDisplayForPoint(
          cursorX,
          cursorY,
          rects,
        );
        final x = (cursorX - winSize.width / 2).clamp(
          originX + 8.0,
          originX + displayW - winSize.width - 8,
        );
        final y = (cursorY + 16).clamp(
          originY + 8.0,
          originY + displayH - winSize.height - 8,
        );
        await windowManager.setSize(winSize);
        // Position before show() to prevent the ghost-flash at the old position.
        await windowManager.setPosition(Offset(x, y));
        await windowManager.setSkipTaskbar(true);
        await windowManager.setAlwaysOnTop(true);
        await windowManager.show();
        if (!Platform.isMacOS) await windowManager.focus();
        await macWindow?.activate();
      } on Object catch (e, st) {
        _log.warning('Picker transition failed, hiding to safe state', e, st);
        try {
          await windowManager.hide();
        } on Object catch (hideErr) {
          _log.warning('hide() after picker failure also failed: $hideErr');
        }
        rethrow;
      }
  }
}

/// Runs before runApp: scan detected browsers and create the icons directory
/// so browsersProvider has data for the first frame.
Future<void> _firstBootEarlyPhase({
  required BrowserService browserService,
  required Directory iconsDir,
}) async {
  await browserService.scanAndMerge();
  try {
    await iconsDir.create(recursive: true);
  } on Object catch (e, st) {
    _log.warning('Could not create icons directory', e, st);
  }
}

/// Runs after runApp: extract icons concurrently and register the app.
/// The picker renders with fallback icons until extraction completes.
Future<void> _firstBootLatePhase({
  required BrowserService browserService,
  required IconExtractor iconExtractor,
  required Directory iconsDir,
  required RegistrationService registrationService,
  required String executablePath,
  required ProviderContainer container,
  bool skipRegistration = false,
}) async {
  // Extract all icons concurrently; swallow per-item errors as before.
  await Future.wait(
    browserService.browsers.map((browser) async {
      try {
        final outputPath =
            '${iconsDir.path}${Platform.pathSeparator}${browser.id}.png';
        await iconExtractor.extractIcon(browser.executablePath, outputPath);
      } on Object catch (e) {
        _log.warning('Icon extraction failed for ${browser.name}: $e');
      }
    }),
  );

  // Invalidate browsersProvider so the picker picks up freshly extracted icons.
  container.invalidate(browsersProvider);

  if (skipRegistration) {
    _log.info('Skipping browser registration in MSIX context');
  } else {
    try {
      await registrationService.register(executablePath);
    } on Object catch (e, st) {
      _log.warning('Browser registration failed (non-fatal)', e, st);
    }
  }
  _log.info('First boot complete: ${browserService.browsers.length} browsers');
}

void _handleUrl(String url, ProviderContainer container) {
  if (looksLikeLocalFile(url)) {
    final resolved = resolveLocalWebFile(url);
    if (resolved == null) {
      _log.warning('Rejected local file: ${_redactForLog(url)}');
      return;
    }
    final fileUri = Uri.file(resolved).toString();
    container.read(appStateProvider.notifier).showPicker(fileUri);
    return;
  }

  final resolved = unwrapSafeLink(url);
  final ruleService = container.read(ruleServiceProvider);
  final matchedBrowserId = ruleService.lookupBrowser(resolved);

  if (matchedBrowserId != null) {
    final browsers = container.read(browserServiceProvider).browsers;
    final browser = browsers.where((b) => b.id == matchedBrowserId).firstOrNull;
    if (browser != null) {
      final launch = container
          .read(launchServiceProvider)
          .launch(browser.executablePath, resolved, browser.extraArgs);
      unawaited(
        launch.catchError((Object e, StackTrace st) {
          _log.severe('Launch failed for ${browser.name}', e, st);
          container.read(appStateProvider.notifier).showPicker(resolved);
        }),
      );
      return;
    }
  }

  container.read(appStateProvider.notifier).showPicker(resolved);
}

String _redactForLog(String raw) {
  if (!looksLikeLocalFile(raw)) return raw;
  if (raw.startsWith('file://')) {
    final uri = Uri.tryParse(raw);
    if (uri == null) return 'file://<unparseable>';
    try {
      return 'file://${redactPath(uri.toFilePath())}';
    } on UnsupportedError {
      return 'file://<unparseable>';
    }
  }
  return redactPath(raw);
}

Future<void> _initTray(
  PlatformBindings bindings,
  ProviderContainer container,
) async {
  await bindings.trayController.init(
    title: 'LinkUnbound',
    iconPath: bindings.trayIconPath,
    tooltip: 'LinkUnbound — Browser Picker',
  );

  bindings.trayController.onActivated(
    () => container.read(appStateProvider.notifier).showSettings(),
  );

  // The tray runs outside the MaterialApp tree, so AppLocalizations.of(context)
  // is not available here; load the configured locale's strings directly.
  final locale = container.read(localeProvider);
  final l10n = await AppLocalizations.delegate.load(
    locale ?? const Locale('en'),
  );

  await bindings.trayController.setMenu([
    TrayMenuItem(
      label: l10n.traySettings,
      onClick: () => container.read(appStateProvider.notifier).showSettings(),
    ),
    const TrayMenuItem.separator(),
    TrayMenuItem(
      label: l10n.exit,
      onClick: () async {
        await container.read(exitAppProvider)();
      },
    ),
  ]);
}
