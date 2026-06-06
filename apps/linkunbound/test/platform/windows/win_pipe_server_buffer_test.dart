import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:linkunbound_core/linkunbound_core.dart';

import 'package:linkunbound/platform/windows/win_pipe_server.dart';
import 'package:linkunbound/platform/windows/windows_bindings.dart';

void main() {
  group('WinPipeServer event buffering (P1.6)', () {
    test('event emitted before first listen is buffered and flushed on listen',
        () async {
      final server = WinPipeServer();

      server.pushEvent(const OpenUrlEvent('https://buffered.example.com'));

      final received = <InboundEvent>[];
      final sub = server.events.listen(received.add);
      addTearDown(sub.cancel);

      await Future<void>.microtask(() {});
      await Future<void>.microtask(() {});

      expect(received, hasLength(1));
      expect(
        received.single,
        isA<OpenUrlEvent>().having(
          (e) => e.url,
          'url',
          'https://buffered.example.com',
        ),
      );
    });

    test(
      'multiple events buffered before first listen are flushed in order',
      () async {
        final server = WinPipeServer();

        server.pushEvent(const OpenUrlEvent('https://first.example.com'));
        server.pushEvent(const OpenUrlEvent('https://second.example.com'));
        server.pushEvent(const ShowSettingsEvent());

        final received = <InboundEvent>[];
        final sub = server.events.listen(received.add);
        addTearDown(sub.cancel);

        await Future<void>.delayed(Duration.zero);

        expect(received, hasLength(3));
        expect(
          received[0],
          isA<OpenUrlEvent>().having(
            (e) => e.url,
            'url',
            'https://first.example.com',
          ),
        );
        expect(
          received[1],
          isA<OpenUrlEvent>().having(
            (e) => e.url,
            'url',
            'https://second.example.com',
          ),
        );
        expect(received[2], isA<ShowSettingsEvent>());
      },
    );

    test(
      'event emitted after first listen is delivered immediately without buffering',
      () async {
        final server = WinPipeServer();

        final received = <InboundEvent>[];
        final sub = server.events.listen(received.add);
        addTearDown(sub.cancel);

        server.pushEvent(const OpenUrlEvent('https://live.example.com'));
        await Future<void>.microtask(() {});

        expect(received, hasLength(1));
        expect(
          received.single,
          isA<OpenUrlEvent>().having(
            (e) => e.url,
            'url',
            'https://live.example.com',
          ),
        );
      },
    );

    test('buffer is not re-flushed when a second listener subscribes', () async {
      final server = WinPipeServer();

      server.pushEvent(const OpenUrlEvent('https://once.example.com'));

      final received1 = <InboundEvent>[];
      final sub1 = server.events.listen(received1.add);
      addTearDown(sub1.cancel);

      await Future<void>.microtask(() {});
      await Future<void>.microtask(() {});

      final received2 = <InboundEvent>[];
      final sub2 = server.events.listen(received2.add);
      addTearDown(sub2.cancel);

      await Future<void>.microtask(() {});

      expect(received1, hasLength(1));
      // Second listener attaches after flush; buffer is already empty.
      expect(received2, isEmpty);
    });
  });

  // -------------------------------------------------------------------------
  // P1.7 — Cross-volume migration fallback
  // -------------------------------------------------------------------------
  group('migrateDirIfNeeded (P1.7)', () {
    late Directory tempDir;

    setUp(() {
      tempDir = Directory.systemTemp.createTempSync('migrate_test_');
    });

    tearDown(() {
      if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
    });

    test('no-op when oldDir does not exist', () {
      final oldDir = Directory('${tempDir.path}/old_missing');
      final newDir = Directory('${tempDir.path}/new');

      migrateDirIfNeeded(oldDir, newDir);

      expect(newDir.existsSync(), isFalse);
    });

    test('no-op when newDir already exists (prevents overwrite)', () {
      final oldDir = Directory('${tempDir.path}/old')..createSync();
      File('${oldDir.path}/data.txt').writeAsStringSync('content');
      final newDir = Directory('${tempDir.path}/new')..createSync();

      migrateDirIfNeeded(oldDir, newDir);

      expect(oldDir.existsSync(), isTrue);
      expect(newDir.listSync(), isEmpty);
    });

    test('rename path: oldDir is moved atomically to newDir', () {
      final oldDir = Directory('${tempDir.path}/old')..createSync();
      File('${oldDir.path}/browsers.json').writeAsStringSync('{}');
      final newDir = Directory('${tempDir.path}/new');

      migrateDirIfNeeded(oldDir, newDir);

      expect(oldDir.existsSync(), isFalse);
      expect(newDir.existsSync(), isTrue);
      expect(File('${newDir.path}/browsers.json').existsSync(), isTrue);
    });

    test('copyDirRecursive copies nested files and deletes source', () {
      final srcDir = Directory('${tempDir.path}/src')..createSync();
      File('${srcDir.path}/browsers.json').writeAsStringSync('{"browsers":[]}');
      final subDir = Directory('${srcDir.path}/icons')..createSync();
      File('${subDir.path}/chrome.png').writeAsBytesSync([1, 2, 3]);

      final dstDir = Directory('${tempDir.path}/dst');
      copyDirRecursive(srcDir, dstDir);

      expect(File('${dstDir.path}/browsers.json').readAsStringSync(), '{"browsers":[]}');
      expect(File('${dstDir.path}/icons/chrome.png').readAsBytesSync(), [1, 2, 3]);
    });

    test('migrateDirIfNeeded preserves data: content matches after move', () {
      final oldDir = Directory('${tempDir.path}/roaming')..createSync();
      File('${oldDir.path}/rules.json').writeAsStringSync('[{"domain":"x.com"}]');
      final newDir = Directory('${tempDir.path}/local');

      migrateDirIfNeeded(oldDir, newDir);

      expect(
        File('${newDir.path}/rules.json').readAsStringSync(),
        '[{"domain":"x.com"}]',
      );
      expect(oldDir.existsSync(), isFalse);
    });
  });
}
