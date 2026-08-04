import 'dart:async';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:linkunbound_core/linkunbound_core.dart';

import 'bootstrap.dart';
import 'platform/macos/macos_bindings.dart';
import 'platform/platform_bindings.dart';
import 'platform/windows/windows_bindings.dart';

Future<void> main(List<String> args) async {
  runZonedGuarded<Future<void>>(
    () async {
      WidgetsFlutterBinding.ensureInitialized();

      FlutterError.onError = (details) {
        _writeStartupCrashLog('FlutterError', details.exception, details.stack);
      };
      PlatformDispatcher.instance.onError = (error, stack) {
        _writeStartupCrashLog('PlatformDispatcher', error, stack);
        return true;
      };

      final PlatformBindings bindings;
      if (Platform.isMacOS) {
        bindings = await MacOsBindings.create();
      } else {
        bindings = await WindowsBindings.create(args);
      }

      await bootstrap(bindings, args);
    },
    (error, stack) {
      _writeStartupCrashLog('runZonedGuarded', error, stack);
    },
  );
}

const _maxCrashLogSize = 256 * 1024;

void _writeStartupCrashLog(String source, Object error, StackTrace? stack) {
  try {
    final String base;
    if (Platform.isWindows) {
      base =
          Platform.environment['LOCALAPPDATA'] ??
          Platform.environment['APPDATA'] ??
          '${Platform.environment['USERPROFILE'] ?? Directory.systemTemp.path}\\AppData\\Local';
    } else {
      base =
          '${Platform.environment['HOME'] ?? Directory.systemTemp.path}/Library/Application Support';
    }
    final dir = Platform.isWindows ? '$base\\LinkUnbound' : '$base/LinkUnbound';
    Directory(dir).createSync(recursive: true);
    final file = File('$dir${Platform.pathSeparator}startup_crash.log');
    // Bounded like navigate.log: this file is append-only and a repeating
    // failure (a browser whose path went stale) would otherwise grow forever.
    if (file.existsSync() && file.lengthSync() > _maxCrashLogSize) {
      file.deleteSync();
    }
    final now = DateTime.now().toIso8601String();
    // Redacted like every other sink: a ProcessException carries the full
    // command line, i.e. the user's URL.
    file.writeAsStringSync(
      '[$now] $source: ${redactUrls('$error')}\n${redactUrls('$stack')}\n\n',
      mode: FileMode.append,
    );
  } on Object {
    // Best-effort crash log; ignore secondary failures.
  }
}
