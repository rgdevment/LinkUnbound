import 'dart:io';

import 'package:logging/logging.dart';

final _log = Logger('MacDiagnosticsService');

const _maxLogLines = 200;

/// macOS counterpart of `exportDiagnostics`. Bundles a system-info report,
/// the LaunchServices URL handler dump for http/https, the rules/browsers
/// JSON snapshots, and the tail of the log into a single zip placed under
/// `appDataDir`. Reveals the result in Finder.
Future<String> exportMacDiagnostics({
  required Directory appDataDir,
  required String appVersion,
}) async {
  final timestamp = DateTime.now()
      .toIso8601String()
      .replaceAll(':', '-')
      .split('.')
      .first;
  final staging = await Directory.systemTemp.createTemp('lu_diag_');

  try {
    _writeSystemInfo(staging, appDataDir, appVersion);
    await _writeLaunchServicesDump(staging);
    _copyDataSnapshots(appDataDir, staging);
    _copyLogTail(appDataDir, staging);

    final zipPath = '${appDataDir.path}/linkunbound-diag-$timestamp.zip';

    // `zip -r -j` flattens directory structure; `-X` strips macOS extras.
    final result = await Process.run('zip', [
      '-r',
      '-j',
      '-X',
      zipPath,
      staging.path,
    ]);

    if (result.exitCode != 0) {
      _log.warning('zip failed: ${result.stderr}');
      throw Exception('zip failed: ${result.stderr}');
    }

    // Reveal in Finder (no-op if user has Finder disabled).
    await Process.run('open', ['-R', zipPath]);

    return zipPath;
  } on Exception catch (e) {
    _log.warning('Diagnostics export failed: $e');
    rethrow;
  } finally {
    try {
      if (staging.existsSync()) await staging.delete(recursive: true);
    } on Exception catch (e) {
      _log.fine('Failed to clean staging dir: $e');
    }
  }
}

void _writeSystemInfo(
  Directory staging,
  Directory appDataDir,
  String appVersion,
) {
  final buf = StringBuffer()
    ..writeln('LinkUnbound Diagnostics Report')
    ..writeln('Generated: ${DateTime.now().toIso8601String()}')
    ..writeln()
    ..writeln('--- System ---')
    ..writeln('OS: ${Platform.operatingSystemVersion}')
    ..writeln('Locale: ${Platform.localeName}')
    ..writeln()
    ..writeln('--- Application ---')
    ..writeln('Version: $appVersion')
    ..writeln('Executable: ${Platform.resolvedExecutable}')
    ..writeln('Data: ${appDataDir.path}')
    ..writeln()
    ..writeln('--- Data Files ---');

  try {
    if (appDataDir.existsSync()) {
      for (final entity in appDataDir.listSync()) {
        final name = entity.path.split('/').last;
        if (entity is File) {
          buf.writeln('  $name (${entity.lengthSync()} bytes)');
        } else if (entity is Directory) {
          buf.writeln('  $name/');
        }
      }
    }
  } on Exception catch (e) {
    buf.writeln('  <error listing files: $e>');
  }

  File('${staging.path}/system_info.txt').writeAsStringSync(buf.toString());
}

Future<void> _writeLaunchServicesDump(Directory staging) async {
  final buf = StringBuffer()
    ..writeln('--- Launch Services (URL handlers) ---')
    ..writeln();

  // `lsappinfo` enumerates running apps; `Launch Services Database` lookups
  // need `lsregister -dump`, but that path is unstable across macOS versions
  // — fall back to recording the bundle ids reported by `defaults read`
  // on the LaunchServices preference plist.
  try {
    final result = await Process.run('defaults', [
      'read',
      'com.apple.LaunchServices/com.apple.launchservices.secure',
      'LSHandlers',
    ]);
    buf.writeln(
      result.exitCode == 0 ? result.stdout : '<unable to read LSHandlers>',
    );
  } on Exception catch (e) {
    buf.writeln('<error: $e>');
  }

  File('${staging.path}/launch_services.txt').writeAsStringSync(buf.toString());
}

void _copyDataSnapshots(Directory appDataDir, Directory staging) {
  for (final name in const ['browsers.json', 'rules.json', 'locale']) {
    final src = File('${appDataDir.path}/$name');
    if (src.existsSync()) {
      try {
        src.copySync('${staging.path}/$name');
      } on Exception catch (e) {
        _log.fine('Failed to copy $name: $e');
      }
    }
  }
}

void _copyLogTail(Directory appDataDir, Directory staging) {
  final logFile = File('${appDataDir.path}/navigate.log');
  if (!logFile.existsSync()) return;

  try {
    final lines = logFile.readAsLinesSync();
    final tail = lines.length > _maxLogLines
        ? lines.sublist(lines.length - _maxLogLines)
        : lines;
    File(
      '${staging.path}/navigate.log',
    ).writeAsStringSync('${tail.join('\n')}\n');
  } on Exception catch (e) {
    _log.fine('Failed to copy log tail: $e');
  }
}
