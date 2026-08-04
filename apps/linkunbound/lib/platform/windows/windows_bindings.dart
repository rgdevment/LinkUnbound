import 'dart:async';
import 'dart:io';
import 'dart:isolate';

import 'package:flutter/foundation.dart';
import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:logging/logging.dart';

import '../cursor_locator.dart';
import '../platform_bindings.dart';
import '../tray_controller.dart';
import 'win_browser_detector.dart';
import 'win_icon_extractor.dart';
import 'win_instance.dart';
import 'win_launch_service.dart';
import 'win_pipe_server.dart';
import 'win_registration_service.dart';
import 'win_source_app.dart';
import 'win_startup_service.dart';
import 'windows_tray_controller.dart';

final _log = Logger('WindowsBindings');
final _windowsAbsPath = RegExp(r'^[a-zA-Z]:[\\/]');

final class WindowsBindings implements PlatformBindings {
  WindowsBindings._({
    required this.browserDetector,
    required this.iconExtractor,
    required this.registrationService,
    required this.startupService,
    required this.launchService,
    required this.trayController,
    required this.cursorLocator,
    required this.appDataDir,
    required this.iconsDir,
    required this.browsersFile,
    required this.rulesFile,
    required this.logFile,
    required this.localeFile,
    required this.themeFile,
    required this.edgeWarningFile,
    required this.hideTrayFile,
    required this.globalHotkeyFile,
    required this.initialEvent,
    required this.startsHidden,
    required WinInstance instance,
    required WinPipeServer pipeServer,
  }) : _instance = instance,
       _pipeServer = pipeServer;

  static Future<WindowsBindings> create(List<String> args) async {
    final baseDir =
        Platform.environment['LOCALAPPDATA'] ??
        Platform.environment['APPDATA'] ??
        '${Platform.environment['USERPROFILE'] ?? Directory.systemTemp.path}\\AppData\\Local';
    final appDataDir = Directory('$baseDir\\LinkUnbound');

    try {
      await appDataDir.create(recursive: true);
    } on FileSystemException catch (e) {
      _log.severe('Could not create app data dir at ${appDataDir.path}', e);
    }

    return WindowsBindings._(
      browserDetector: WinBrowserDetector(),
      iconExtractor: WinIconExtractor(),
      registrationService: WinRegistrationService(),
      startupService: WinStartupService(),
      launchService: WinLaunchService(),
      trayController: WindowsTrayController(),
      cursorLocator: const ScreenRetrieverCursorLocator(),
      appDataDir: appDataDir,
      iconsDir: Directory('${appDataDir.path}\\icons'),
      browsersFile: File('${appDataDir.path}\\browsers.json'),
      rulesFile: File('${appDataDir.path}\\rules.json'),
      logFile: File('${appDataDir.path}\\navigate.log'),
      localeFile: File('${appDataDir.path}\\locale'),
      themeFile: File('${appDataDir.path}\\theme'),
      edgeWarningFile: File('${appDataDir.path}\\edge_warning_dismissed'),
      hideTrayFile: File('${appDataDir.path}\\hide_tray'),
      globalHotkeyFile: File('${appDataDir.path}\\global_hotkey'),
      initialEvent: _parseInitialEvent(args),
      startsHidden: args.contains('--background'),
      instance: WinInstance(),
      pipeServer: WinPipeServer(),
    );
  }

  @override
  final BrowserDetector browserDetector;
  @override
  final IconExtractor iconExtractor;
  @override
  final RegistrationService registrationService;
  @override
  final StartupService startupService;
  @override
  final LaunchService launchService;
  @override
  final TrayController trayController;
  @override
  final CursorLocator cursorLocator;
  @override
  final Directory appDataDir;
  @override
  final Directory iconsDir;
  @override
  final File browsersFile;
  @override
  final File rulesFile;
  @override
  final File logFile;
  @override
  final File localeFile;
  @override
  final File themeFile;
  @override
  final File edgeWarningFile;
  @override
  final File hideTrayFile;
  @override
  final File globalHotkeyFile;
  @override
  final InboundEvent? initialEvent;
  @override
  final bool startsHidden;

  final WinInstance _instance;
  final WinPipeServer _pipeServer;

  @override
  String get executablePath => Platform.resolvedExecutable;

  @override
  String get trayIconPath => 'assets/app_icon.ico';

  @override
  Stream<InboundEvent> get inboundEvents {
    final initial = initialEvent;
    // Push into the server's own buffer instead of wrapping the stream. An
    // `async*` wrapper evaluated `_pipeServer.events` eagerly — which marks
    // the buffer as drained — but only subscribed to the broadcast controller
    // a microtask later, so anything flushed in between was dropped on the
    // floor. That window is exactly a cold start handling a link.
    if (initial != null) _pipeServer.pushEvent(initial);
    return _pipeServer.events;
  }

  @override
  Future<bool> tryDelegate(InboundEvent? event) async {
    final client = WinPipeClient();
    final payload = event ?? const ShowSettingsEvent();
    WinInstance.allowForeground();
    // Retry with backoff: resident may have the mutex but the pipe may not be
    // bound yet (TOCTOU window between acquire() and the isolate calling
    // ConnectNamedPipe).
    for (var attempt = 0; attempt < 3; attempt++) {
      if (await client.send(payload)) return true;
      if (attempt < 2) {
        await Future<void>.delayed(const Duration(milliseconds: 50));
      }
    }
    return false;
  }

  @override
  Future<bool> claim() async {
    if (!_instance.acquire()) {
      _log.warning('Mutex held but pipe unreachable');
      return false;
    }
    try {
      await _pipeServer.start();
      // Wait until the pipe server is actually listening before returning so
      // that a concurrent tryDelegate() from an incoming process can connect
      // without hitting the TOCTOU race window.
      await _pipeServer.ready.timeout(
        const Duration(seconds: 5),
        onTimeout: () {
          // We continue as the resident instance because the picker still works
          // locally; only cross-process delegation (secondary instances sending
          // URLs) will silently fail until the pipe eventually becomes ready.
          _log.severe(
            'Pipe server readiness timeout: secondary instances will not be '
            'able to delegate URLs to this instance until the pipe is ready.',
          );
        },
      );
    } on Exception catch (e) {
      _log.warning('Pipe server failed to start: $e');
    }
    // Run in a worker isolate: cross-volume migration copies files and would
    // block the UI thread.
    final newPath = appDataDir.path;
    try {
      await Isolate.run(() => _migrateFromRoamingIfNeeded(Directory(newPath)));
    } on Object catch (e) {
      _log.warning('App data migration failed: $e');
    }
    return true;
  }

  @override
  Future<void> release() async {
    await _pipeServer.stop();
    _instance.release();
  }

  static InboundEvent? _parseInitialEvent(List<String> args) {
    for (final arg in args) {
      if (arg.startsWith('--')) continue;
      final resolved = stripEdgeProtocol(arg);
      final lower = resolved.toLowerCase();
      // Textual check first: Uri.tryParse returns null for URLs that are
      // malformed but perfectly real (unescaped brackets or stray `%` show up
      // routinely in Teams and SharePoint links), and those were silently
      // dropped.
      if (lower.startsWith('http://') || lower.startsWith('https://')) {
        return OpenUrlEvent(resolved, sourceApp: _sourceApp());
      }
      final uri = Uri.tryParse(resolved);
      if (uri != null && uri.scheme.toLowerCase() == 'file') {
        return OpenUrlEvent(resolved, sourceApp: _sourceApp());
      }
      if (_windowsAbsPath.hasMatch(arg)) {
        return OpenUrlEvent(arg, sourceApp: _sourceApp());
      }
      // Without this line a dropped link leaves no trace at all, which is why
      // the failure was so hard to diagnose in the field.
      _log.warning(
        'Ignoring unrecognised launch argument (scheme=${uri?.scheme})',
      );
    }
    return null;
  }

  /// The app that asked the shell to open this link.
  ///
  /// Only meaningful in the process the shell just launched: once the URL is
  /// delegated to the resident instance over the pipe, that instance's parent
  /// is unrelated. Hence resolving it here, at parse time, and shipping it
  /// inside the event.
  static String? _sourceApp() {
    final name = parentProcessName();
    // The shell itself is not a useful origin to write rules against.
    if (name == null ||
        const {'explorer', 'cmd', 'powershell'}.contains(name)) {
      return null;
    }
    return name;
  }

  static void _migrateFromRoamingIfNeeded(Directory newDir) {
    final roamingBase = Platform.environment['APPDATA'];
    if (roamingBase == null || roamingBase.isEmpty) return;
    final oldDir = Directory('$roamingBase\\LinkUnbound');
    migrateDirIfNeeded(oldDir, newDir);
  }
}

/// Moves [oldDir] to [newDir] atomically when possible; falls back to
/// recursive copy + delete when they are on different volumes or when the
/// destination directory already exists.
@visibleForTesting
void migrateDirIfNeeded(Directory oldDir, Directory newDir) {
  if (!oldDir.existsSync()) return;
  // Startup creates newDir before migration runs, so only real user data
  // (browsers.json, same marker as bootstrap's first-boot check) can gate it.
  final dataMarker = File(
    '${newDir.path}${Platform.pathSeparator}browsers.json',
  );
  if (dataMarker.existsSync()) return;

  if (!newDir.existsSync()) {
    try {
      oldDir.renameSync(newDir.path);
      _log.info('Migrated app data from ${oldDir.path} to ${newDir.path}');
      return;
    } on FileSystemException {
      // rename fails across volumes; fall through to copy + delete.
    }
  }

  _log.info('Cross-volume migration: copying ${oldDir.path} to ${newDir.path}');
  try {
    copyDirRecursive(oldDir, newDir);
    oldDir.deleteSync(recursive: true);
    _log.info('Cross-volume migration complete');
  } on FileSystemException catch (e) {
    _log.warning('Cross-volume migration failed: $e');
    // Leave oldDir intact so the user doesn't lose data.
  }
}

@visibleForTesting
void copyDirRecursive(Directory src, Directory dst) {
  dst.createSync(recursive: true);
  for (final entity in src.listSync()) {
    final name = entity.uri.pathSegments.lastWhere((s) => s.isNotEmpty);
    final target = '${dst.path}${Platform.pathSeparator}$name';
    if (entity is File) {
      // Existing files belong to this boot (e.g. the open log file, which
      // Windows refuses to overwrite); skipping them keeps the migration alive.
      if (!File(target).existsSync()) {
        entity.copySync(target);
      }
    } else if (entity is Directory) {
      copyDirRecursive(entity, Directory(target));
    }
  }
}
