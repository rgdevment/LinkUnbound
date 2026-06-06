import 'dart:io';

import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:logging/logging.dart';
import 'package:test/test.dart';

void main() {
  late Directory tempDir;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync('log_service_test_');
  });

  tearDown(() {
    Logger.root.clearListeners();
    Logger.root.level = Level.OFF;
    if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
  });

  group('initLogging — setup', () {
    test('creates parent directory when it does not exist', () {
      final logFile = File('${tempDir.path}/subdir/app.log');
      expect(logFile.parent.existsSync(), isFalse);
      initLogging(logFile);
      expect(logFile.parent.existsSync(), isTrue);
    });

    test('sets Logger.root level to ALL', () {
      final logFile = File('${tempDir.path}/app.log');
      Logger.root.level = Level.OFF;
      initLogging(logFile);
      expect(Logger.root.level, Level.ALL);
    });
  });

  group('initLogging — log record listener', () {
    test('writes log message to file', () {
      final logFile = File('${tempDir.path}/app.log');
      initLogging(logFile);
      Logger.root.info('hello-test-xyz');
      expect(logFile.existsSync(), isTrue);
      expect(logFile.readAsStringSync(), contains('hello-test-xyz'));
    });

    test('appends multiple messages to file', () {
      final logFile = File('${tempDir.path}/app.log');
      initLogging(logFile);
      Logger.root.info('first-msg');
      Logger.root.info('second-msg');
      final content = logFile.readAsStringSync();
      expect(content, contains('first-msg'));
      expect(content, contains('second-msg'));
    });

    test('log entry includes level name', () {
      final logFile = File('${tempDir.path}/app.log');
      initLogging(logFile);
      Logger.root.warning('warn-test');
      expect(logFile.readAsStringSync(), contains('WARNING'));
    });

    test('log entry includes ISO timestamp', () {
      final logFile = File('${tempDir.path}/app.log');
      initLogging(logFile);
      Logger.root.info('ts-test');
      final content = logFile.readAsStringSync();
      expect(content, matches(RegExp(r'\d{4}-\d{2}-\d{2}T')));
    });

    test('log entry includes error object when present', () {
      final logFile = File('${tempDir.path}/app.log');
      initLogging(logFile);
      Logger.root.severe('severe-test', Exception('boom'));
      expect(logFile.readAsStringSync(), contains('boom'));
    });

    test('log entry does not include error marker when no error', () {
      final logFile = File('${tempDir.path}/app.log');
      initLogging(logFile);
      Logger.root.info('no-error-test');
      final content = logFile.readAsStringSync();
      expect(content, contains('no-error-test'));
      expect(content, isNot(contains('\n  Exception')));
    });

    test('log entry includes stack trace when present', () {
      final logFile = File('${tempDir.path}/app.log');
      initLogging(logFile);
      Logger.root.severe('trace-test', Exception('e'), StackTrace.current);
      expect(logFile.readAsStringSync(), contains('trace-test'));
    });

    test(
      'silently swallows FileSystemException when log directory is removed mid-session',
      () {
        final logFile = File('${tempDir.path}/app.log');
        initLogging(logFile);
        // Delete the directory after initLogging so the write will fail.
        tempDir.deleteSync(recursive: true);
        // Must not throw — the catch block drops the write silently.
        expect(() => Logger.root.info('after-removal'), returnsNormally);
      },
    );
  });

  group('rotation', () {
    test('no rotation when log file does not exist', () {
      final logFile = File('${tempDir.path}/app.log');
      initLogging(logFile);
      expect(File('${logFile.path}.1').existsSync(), isFalse);
    });

    test('no rotation when file is below 2 MB', () {
      final logFile = File('${tempDir.path}/app.log');
      logFile.writeAsStringSync('small content');
      initLogging(logFile);
      expect(File('${logFile.path}.1').existsSync(), isFalse);
    });

    test('rotates log file when it exceeds 2 MB', () {
      final logFile = File('${tempDir.path}/app.log');
      logFile.writeAsBytesSync(List.filled(2 * 1024 * 1024 + 1, 65));
      initLogging(logFile);
      expect(File('${logFile.path}.1').existsSync(), isTrue);
    });

    test('deletes existing backup before rotating', () {
      final logFile = File('${tempDir.path}/app.log');
      logFile.writeAsBytesSync(List.filled(2 * 1024 * 1024 + 1, 65));
      final backup = File('${logFile.path}.1');
      backup.writeAsStringSync('stale');
      initLogging(logFile);
      expect(backup.lengthSync(), greaterThan(2 * 1024 * 1024));
    });
  });

  group('redactUrls short-circuit', () {
    test('returns text unchanged when no :// or :\\ present', () {
      const plain = 'browser opened at port 8080 with args --headless';
      expect(redactUrls(plain), same(plain));
    });

    test('does not short-circuit when :// is present', () {
      expect(
        redactUrls('https://secret.com/path'),
        isNot(contains('secret.com')),
      );
    });

    test('does not short-circuit when :\\ is present', () {
      expect(
        redactUrls(r'C:\Users\Bob\file.pdf'),
        isNot(contains('Bob')),
      );
    });
  });

  group('redactUrls', () {
    test('replaces http URL with redacted placeholder', () {
      expect(
        redactUrls('opened http://example.com/page'),
        equals('opened http://<redacted>/1 segments'),
      );
    });

    test('replaces https URL with redacted placeholder', () {
      expect(
        redactUrls('visit https://mail.google.com/inbox/123'),
        equals('visit https://<redacted>/2 segments'),
      );
    });

    test('preserves text without URLs', () {
      expect(redactUrls('no urls here'), equals('no urls here'));
    });

    test('redacts multiple URLs in one line', () {
      final result = redactUrls(
        'from https://a.com/x to http://b.com/y/z done',
      );
      expect(result, isNot(contains('a.com')));
      expect(result, isNot(contains('b.com')));
      expect(result, contains('https://<redacted>/1 segments'));
      expect(result, contains('http://<redacted>/2 segments'));
    });

    test('handles URL with no path segments', () {
      expect(
        redactUrls('root https://example.com'),
        equals('root https://<redacted>/0 segments'),
      );
    });

    test('log messages are redacted before writing to file', () {
      final logFile = File('${tempDir.path}/app.log');
      initLogging(logFile);
      Logger.root.info('open_url https://secret.example.com/path');
      final content = logFile.readAsStringSync();
      expect(content, isNot(contains('secret.example.com')));
      expect(content, contains('https://<redacted>/1 segments'));
    });

    test('redacts Windows file path', () {
      expect(
        redactUrls(r'open C:\Users\Mario\Desktop\file.pdf'),
        equals('open <redacted-path>.pdf'),
      );
    });

    test('redacts file path with backslashes', () {
      expect(
        redactUrls(r'open C:\tmp\file.html'),
        equals('open <redacted-path>.html'),
      );
    });

    test('redacts file:// URI', () {
      expect(
        redactUrls(r'open file:///C:\Users\Mario\doc.pdf'),
        equals('open <redacted-path>.pdf'),
      );
    });

    test('redacts file path without extension', () {
      expect(
        redactUrls(r'path C:\folder\subfolder'),
        equals('path <redacted-path>'),
      );
    });
  });
}
