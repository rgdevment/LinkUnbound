import 'dart:async';
import 'dart:io';

import 'package:logging/logging.dart';

const _maxLogSize = 2 * 1024 * 1024; // 2 MB

StreamSubscription<LogRecord>? _logSubscription;

// Kept open for the session lifetime to avoid reopening the fd on every record.
RandomAccessFile? _logRaf;

final _urlPattern = RegExp(r'https?://[^\s,\]\)]+', caseSensitive: false);
final _filePathPattern = RegExp(
  r'file:///[a-zA-Z]:[\\/][^\s,\]\)]*|[a-zA-Z]:\\[^\s,\]\)]*',
  caseSensitive: false,
);

String redactUrls(String text) {
  // Skip regex passes when no URL/path sentinel character is present.
  if (!text.contains('://') && !text.contains(':\\')) return text;
  var result = text.replaceAllMapped(_urlPattern, (match) {
    final url = match.group(0)!;
    final uri = Uri.tryParse(url);
    if (uri == null) return '<redacted-url>';
    return '${uri.scheme}://<redacted>/${uri.pathSegments.length} segments';
  });
  result = result.replaceAllMapped(_filePathPattern, (match) {
    final path = match.group(0)!;
    final ext = RegExp(r'\.[a-zA-Z0-9]+$').firstMatch(path);
    return '<redacted-path>${ext?[0] ?? ''}';
  });
  return result;
}

void initLogging(File logFile, {Level fileLevel = Level.INFO}) {
  _logSubscription?.cancel();
  _logSubscription = null;
  // Close the previous file handle before (re)opening to avoid fd leaks.
  _logRaf?.closeSync();
  _logRaf = null;

  try {
    logFile.parent.createSync(recursive: true);
    _rotateIfNeeded(logFile);
    _logRaf = logFile.openSync(mode: FileMode.append);
  } on FileSystemException {
    // Best-effort: file logging will be disabled below if writes fail.
  }

  Logger.root.level = Level.ALL;
  // Windows GUI subsystem binaries (flutter build windows --release) have no
  // valid stdio handles, and even `stderr.hasTerminal` can lie; a single
  // write then crashes the main zone asynchronously. On Windows we therefore
  // skip console output entirely and rely on the file sink.
  var useStderr = !Platform.isWindows;
  _logSubscription = Logger.root.onRecord.listen((record) {
    if (record.level < fileLevel) return;
    final message = redactUrls(record.message);
    final line =
        '${record.time.toIso8601String()} '
        '[${record.level.name}] '
        '${record.loggerName}: '
        '$message'
        '${record.error != null ? '\n  ${record.error}' : ''}'
        '${record.stackTrace != null ? '\n  ${record.stackTrace}' : ''}';
    if (useStderr) {
      try {
        stderr.writeln(line);
      } on Object {
        useStderr = false;
      }
    }
    final raf = _logRaf;
    if (raf != null) {
      try {
        raf.writeStringSync('$line\n');
      } on FileSystemException {
        // Log directory may have been removed (e.g. during tests). Drop silently.
      }
    }
  });
}

void _rotateIfNeeded(File logFile) {
  if (!logFile.existsSync()) return;
  if (logFile.lengthSync() < _maxLogSize) return;

  final backup = File('${logFile.path}.1');
  if (backup.existsSync()) backup.deleteSync();
  logFile.renameSync(backup.path);
}
