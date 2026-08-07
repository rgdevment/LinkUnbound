import 'dart:async';
import 'dart:io';
import 'dart:ui' show PlatformDispatcher;

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
import 'platform/macos/mac_source_app.dart';
import 'platform/macos/mac_window_channel.dart';
import 'platform/platform_bindings.dart';
import 'platform/tray_controller.dart';
import 'platform/windows/win_package_context.dart';
import 'providers.dart';
import 'ui/picker/picker_layout.dart';

final _log = Logger('Bootstrap');

Never _exitAfterFlush() {
  disposeLogging();
  exit(0);
}

Future<void> bootstrap(
  PlatformBindings bindings,
  List<String> args, {
  Never Function() exitProcess = _exitAfterFlush,
}) async {
  initLogging(bindings.logFile);

  _log.info('LinkUnbound starting (msix=${isRunningInMsix()})');

  try {
    await bindings.registrationService.ensureRegistered(
      bindings.executablePath,
    );
  } on Object catch (e, st) {
    _log.warning('Registration reconciliation failed (non-fatal)', e, st);
  }

  if (args.contains('--register')) {
    _log.info('Registration-only run; exiting without starting the UI');
    exitProcess();
  }

  Future<bool> delegate(String failureMessage) async {
    try {
      return await bindings.tryDelegate(bindings.initialEvent);
    } on Object catch (e, st) {
      _log.warning(failureMessage, e, st);
      return false;
    }
  }

  if (await delegate('Delegation check failed')) exitProcess();

  bool claimed;
  try {
    claimed = await bindings.claim();
  } on Object catch (e, st) {
    _log.severe('claim() crashed', e, st);
    claimed = false;
  }

  if (!claimed) {
    if (await delegate('Post-claim delegation retry failed')) exitProcess();
    try {
      claimed = await bindings.claim();
    } on Object catch (e, st) {
      _log.severe('Second claim() attempt crashed', e, st);
      claimed = false;
    }
    if (!claimed) {
      if (await delegate('Final delegation attempt failed')) exitProcess();
      final eventType = bindings.initialEvent?.runtimeType;
      if (eventType != null) {
        _log.severe(
          'Discarding initial event: no resident could be reached '
          '(type=$eventType)',
        );
      }
      exitProcess();
    }
  }

  final browserService = BrowserService(
    configFile: bindings.browsersFile,
    browserDetector: bindings.browserDetector,
  );
  final ruleService = RuleService(rulesFile: bindings.rulesFile);

  var isFirstBoot = !bindings.browsersFile.existsSync();

  try {
    await browserService.load();
  } on Object catch (e, st) {
    _log.severe('Browser config corrupted, resetting', e, st);
    try {
      await browserService.reset();
      isFirstBoot = true;
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
        backgroundColor: Color(0xFF1E1E1E),
      ),
    );
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
  } on Object catch (e, st) {
    _log.severe('Window manager init failed', e, st);
  }

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
      themeFileProvider.overrideWithValue(bindings.themeFile),
      edgeWarningFileProvider.overrideWithValue(bindings.edgeWarningFile),
      hideTrayFileProvider.overrideWithValue(bindings.hideTrayFile),
      globalHotkeyFileProvider.overrideWithValue(bindings.globalHotkeyFile),
      appDataDirProvider.overrideWithValue(bindings.appDataDir),
      executablePathProvider.overrideWithValue(bindings.executablePath),
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
        exitProcess();
      }),
    ],
  );

  final macWindow = Platform.isMacOS ? MacWindowChannel() : null;

  AppState? applied;
  var applying = false;

  Future<void> drainModeChanges() async {
    if (applying) return;
    applying = true;
    try {
      var target = container.read(appStateProvider);
      while (!identical(target, applied)) {
        final previous = applied;
        applied = target;
        try {
          // A transition that never completes must not deafen the app to
          // every link that follows.
          await _applyAppMode(
            previous,
            target,
            container,
            bindings,
            macWindow,
          ).timeout(const Duration(seconds: 10));
        } on Object catch (e, st) {
          _log.warning('App mode transition failed', e, st);
        }
        target = container.read(appStateProvider);
      }
    } finally {
      applying = false;
    }
  }

  container.listen<AppState>(appStateProvider, (prev, next) {
    unawaited(drainModeChanges());
  });

  // Subscribe to inbound events before runApp so no event is dropped while
  // Flutter initialises. The ready signal fires only when this listener
  // attaches (see MacInboundEvents._signalReadyOnce).
  bindings.inboundEvents.listen(
    (event) {
      try {
        switch (event) {
          case OpenUrlEvent(:final url, :final sourceApp):
            unawaited(_handleUrl(url, container, sourceApp: sourceApp));
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
    // A launch that carries a URL is a link click, not a request to open
    // Settings; opening it anyway queues a settings→picker transition pair.
    if (bindings.initialEvent is OpenUrlEvent) return;
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
    switch (next.mode) {
      case AppMode.settings:
        await windowManager.show();
        await windowManager.focus();
        await macWindow?.activate();
      case AppMode.picker:
        // A second link while the picker is already up must reposition and
        // re-show it. Returning early here used to make the app go deaf to
        // every subsequent link once a transition had failed mid-way.
        await _showPicker(container, bindings, macWindow);
      case AppMode.hidden:
        break;
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
      await windowManager.setSize(Size(640, Platform.isMacOS ? 728 : 700));
      await windowManager.center();
      await windowManager.setSkipTaskbar(false);
      await windowManager.setAlwaysOnTop(false);
      await windowManager.show();
      await windowManager.focus();
      await macWindow?.activate();
    case AppMode.picker:
      await _showPicker(container, bindings, macWindow);
  }
}

Future<void> _showPicker(
  ProviderContainer container,
  PlatformBindings bindings,
  MacWindowChannel? macWindow,
) async {
  try {
    await macWindow?.setPickerMode();
    final browsers = container.read(browsersProvider);
    final winSize = PickerLayout.windowSize(
      browsers.length,
      textScale: PlatformDispatcher.instance.textScaleFactor,
    );
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
    final x = _clampToRange(
      cursorX - winSize.width / 2,
      originX + 8.0,
      originX + displayW - winSize.width - 8,
    );
    final y = _clampToRange(
      cursorY + 16,
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
    await windowManager.setSize(winSize);
  } on Object catch (e, st) {
    _log.warning('Picker transition failed, returning to hidden', e, st);
    try {
      await windowManager.hide();
    } on Object catch (hideErr) {
      _log.warning('hide() after picker failure also failed: $hideErr');
    }
    container.read(appStateProvider.notifier).hide();
    rethrow;
  }
}

double _clampToRange(double value, double lower, double upper) =>
    upper < lower ? lower : value.clamp(lower, upper).toDouble();

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

Future<void> _firstBootLatePhase({
  required BrowserService browserService,
  required IconExtractor iconExtractor,
  required Directory iconsDir,
  required ProviderContainer container,
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

  _log.info('First boot complete: ${browserService.browsers.length} browsers');
}

Future<void> _handleUrl(
  String url,
  ProviderContainer container, {
  String? sourceApp,
}) async {
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
  if (!isLaunchableUrl(resolved)) {
    _log.warning('Rejected URL with non-launchable scheme');
    return;
  }

  final origin = sourceApp ?? (Platform.isMacOS ? await _macSourceApp() : null);
  if (origin != null) _log.fine('Link originated from $origin');

  final ruleService = container.read(ruleServiceProvider);
  final rule = ruleService.lookupRule(resolved, sourceApp: origin);

  if (rule != null) {
    final browsers = container.read(browserServiceProvider).browsers;
    final browser = browsers.where((b) => b.id == rule.browserId).firstOrNull;
    if (browser != null) {
      final launch = container
          .read(launchServiceProvider)
          .launch(
            browser.executablePath,
            resolved,
            browser.extraArgs,
            privateArgs: rule.private ? browser.resolvedPrivateArgs : const [],
          );
      unawaited(
        launch.catchError((Object e, StackTrace st) {
          _log.severe('Launch failed for ${browser.name}', e, st);
          container.read(appStateProvider.notifier).showPicker(resolved);
        }),
      );
      return;
    }
  }

  container
      .read(appStateProvider.notifier)
      .showPicker(resolved, origin: origin);
}

Future<String?> _macSourceApp() async {
  try {
    return (await frontmostApp())?.id;
  } on Object catch (e) {
    _log.fine('Frontmost app lookup failed: $e');
    return null;
  }
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
