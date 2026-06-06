import 'dart:io';

import 'package:archive/archive_io.dart';
import 'package:logging/logging.dart';
import 'package:win32_registry/win32_registry.dart';

final _log = Logger('DiagnosticsService');

const _maxLogLines = 200;

const _registryPaths = [
  r'Software\Classes\LinkUnboundURL',
  r'Software\Clients\StartMenuInternet\LinkUnbound',
  r'Software\LinkUnbound',
];

Future<String> exportDiagnostics({
  required Directory appDataDir,
  required String appVersion,
  void Function(Directory staging)? registryDumper,
  Future<void> Function(String zipPath)? revealer,
}) async {
  final timestamp = DateTime.now()
      .toIso8601String()
      .replaceAll(':', '-')
      .split('.')
      .first;
  final staging = await Directory.systemTemp.createTemp('lu_diag_');

  try {
    _writeSystemInfo(staging, appDataDir, appVersion);
    (registryDumper ?? _writeRegistryDump)(staging);
    _copyLogTail(appDataDir, staging);

    final zipPath =
        '${appDataDir.path}${Platform.pathSeparator}linkunbound-diag-$timestamp.zip';

    try {
      final encoder = ZipFileEncoder()..create(zipPath);
      for (final entity in staging.listSync(recursive: true)) {
        if (entity is File) {
          await encoder.addFile(
            entity,
            entity.path
                .substring(staging.path.length + 1)
                .replaceAll('\\', '/'),
          );
        }
      }
      await encoder.close();
    } on Object catch (e) {
      _log.warning('Diagnostics zip creation failed: $e');
      throw Exception('Diagnostics zip creation failed: $e');
    }

    try {
      await (revealer ?? _revealInExplorer)(zipPath);
    } on ProcessException catch (e) {
      _log.fine('Could not reveal zip in Explorer: ${e.message}');
    }

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

Future<void> _revealInExplorer(String zipPath) async {
  await Process.run('explorer.exe', ['/select,$zipPath']);
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
    ..writeln('OS: ${_osVersion()}')
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
        final name = entity.uri.pathSegments.where((s) => s.isNotEmpty).last;
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

  File(
    '${staging.path}${Platform.pathSeparator}system_info.txt',
  ).writeAsStringSync(buf.toString());
}

String _osVersion() => parseWindowsVersion(Platform.operatingSystemVersion);

String parseWindowsVersion(String raw) {
  final buildMatch = RegExp(r'Build (\d+)').firstMatch(raw);
  if (buildMatch == null) return raw;
  final build = int.tryParse(buildMatch.group(1)!) ?? 0;
  if (build >= 22000) return raw.replaceFirst('Windows 10', 'Windows 11');
  return raw;
}

void _writeRegistryDump(Directory staging) {
  final buf = StringBuffer()
    ..writeln('Registry Dump')
    ..writeln('Generated: ${DateTime.now().toIso8601String()}')
    ..writeln();

  for (final path in _registryPaths) {
    _dumpKey(path, buf);
    buf.writeln();
  }

  File(
    '${staging.path}${Platform.pathSeparator}registry.txt',
  ).writeAsStringSync(buf.toString());
}

void _dumpKey(String path, StringBuffer buf, {int depth = 0}) {
  final indent = '  ' * depth;
  try {
    final key = Registry.openPath(RegistryHive.currentUser, path: path);

    buf.writeln('$indent[HKCU\\$path]');
    for (final v in key.values) {
      final name = v.name.isEmpty ? '(Default)' : v.name;
      try {
        buf.writeln('$indent  $name = ${v.data} [${v.type}]');
      } on Exception {
        buf.writeln('$indent  $name = <unreadable> [${v.type}]');
      }
    }

    final subkeys = key.subkeyNames.toList();
    key.close();

    for (final sub in subkeys) {
      _dumpKey('$path\\$sub', buf, depth: depth + 1);
    }
  } on Exception {
    buf.writeln('$indent[HKCU\\$path] — not found');
  }
}

void _copyLogTail(Directory appDataDir, Directory staging) {
  final logFile = File(
    '${appDataDir.path}${Platform.pathSeparator}navigate.log',
  );
  if (!logFile.existsSync()) return;

  try {
    final lines = logFile.readAsLinesSync();
    final tail = lines.length > _maxLogLines
        ? lines.sublist(lines.length - _maxLogLines)
        : lines;

    File(
      '${staging.path}${Platform.pathSeparator}navigate.log',
    ).writeAsStringSync(tail.join('\n'));
  } on Exception catch (e) {
    File(
      '${staging.path}${Platform.pathSeparator}navigate.log',
    ).writeAsStringSync('<error reading log: $e>');
  }
}
