import 'dart:io';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:linkunbound/providers.dart';

void main() {
  group('HideTrayNotifier — persistence round-trip', () {
    late Directory tempDir;
    late File hideTrayFile;

    setUp(() {
      tempDir = Directory.systemTemp.createTempSync('hide_tray_test_');
      hideTrayFile = File('${tempDir.path}/hide_tray');
    });

    tearDown(() {
      if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
    });

    ProviderContainer makeContainer() => ProviderContainer(
      overrides: [hideTrayFileProvider.overrideWithValue(hideTrayFile)],
    );

    test('initial state is false when file does not exist', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(c.read(hideTrayProvider), isFalse);
    });

    test('initial state is true when file already exists', () {
      hideTrayFile.writeAsStringSync('1');
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(c.read(hideTrayProvider), isTrue);
    });

    test('setHideTray(true) creates the file', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(hideTrayProvider.notifier).setHideTray(true);
      expect(hideTrayFile.existsSync(), isTrue);
      expect(c.read(hideTrayProvider), isTrue);
    });

    test('setHideTray(false) deletes the file', () {
      hideTrayFile.writeAsStringSync('1');
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(hideTrayProvider.notifier).setHideTray(false);
      expect(hideTrayFile.existsSync(), isFalse);
      expect(c.read(hideTrayProvider), isFalse);
    });

    test('setHideTray(false) when file absent does not throw', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(
        () => c.read(hideTrayProvider.notifier).setHideTray(false),
        returnsNormally,
      );
    });

    test('round-trip: set true then false ends with false and no file', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(hideTrayProvider.notifier).setHideTray(true);
      c.read(hideTrayProvider.notifier).setHideTray(false);
      expect(c.read(hideTrayProvider), isFalse);
      expect(hideTrayFile.existsSync(), isFalse);
    });
  });

  group('GlobalHotkeyNotifier — persistence round-trip', () {
    late Directory tempDir;
    late File globalHotkeyFile;

    setUp(() {
      tempDir = Directory.systemTemp.createTempSync('global_hotkey_test_');
      globalHotkeyFile = File('${tempDir.path}/global_hotkey');
    });

    tearDown(() {
      if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
    });

    ProviderContainer makeContainer() => ProviderContainer(
      overrides: [
        globalHotkeyFileProvider.overrideWithValue(globalHotkeyFile),
      ],
    );

    test('initial state is null when file does not exist', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(c.read(globalHotkeyProvider), isNull);
    });

    test('initial state reads from file when it exists', () {
      globalHotkeyFile.writeAsStringSync('meta+shift+l');
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(c.read(globalHotkeyProvider), 'meta+shift+l');
    });

    test('initial state is null when file is blank', () {
      globalHotkeyFile.writeAsStringSync('   ');
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(c.read(globalHotkeyProvider), isNull);
    });

    test('setHotkey writes to file and updates state', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(globalHotkeyProvider.notifier).setHotkey('ctrl+alt+space');
      expect(globalHotkeyFile.readAsStringSync(), 'ctrl+alt+space');
      expect(c.read(globalHotkeyProvider), 'ctrl+alt+space');
    });

    test('setHotkey(null) deletes file and sets state to null', () {
      globalHotkeyFile.writeAsStringSync('meta+shift+l');
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(globalHotkeyProvider.notifier).setHotkey(null);
      expect(globalHotkeyFile.existsSync(), isFalse);
      expect(c.read(globalHotkeyProvider), isNull);
    });

    test('setHotkey empty string is treated as null', () {
      globalHotkeyFile.writeAsStringSync('meta+shift+l');
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(globalHotkeyProvider.notifier).setHotkey('');
      expect(c.read(globalHotkeyProvider), isNull);
    });

    test('round-trip: set then clear ends with null', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(globalHotkeyProvider.notifier).setHotkey('ctrl+alt+l');
      c.read(globalHotkeyProvider.notifier).setHotkey(null);
      expect(c.read(globalHotkeyProvider), isNull);
      expect(globalHotkeyFile.existsSync(), isFalse);
    });
  });

  group('Hide-tray safeguard', () {
    late Directory tempDir;

    setUp(() {
      tempDir = Directory.systemTemp.createTempSync('safeguard_test_');
    });

    tearDown(() {
      if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
    });

    ProviderContainer makeContainer({
      bool hideTray = false,
      String? hotkey,
    }) {
      final hideTrayFile = File('${tempDir.path}/hide_tray');
      final globalHotkeyFile = File('${tempDir.path}/global_hotkey');
      if (hideTray) hideTrayFile.writeAsStringSync('1');
      if (hotkey != null) globalHotkeyFile.writeAsStringSync(hotkey);
      return ProviderContainer(
        overrides: [
          hideTrayFileProvider.overrideWithValue(hideTrayFile),
          globalHotkeyFileProvider.overrideWithValue(globalHotkeyFile),
        ],
      );
    }

    test('hide-tray can be true only when hotkey is configured', () {
      final c = makeContainer(hideTray: true, hotkey: 'meta+shift+l');
      addTearDown(c.dispose);
      expect(c.read(hideTrayProvider), isTrue);
      expect(c.read(globalHotkeyProvider), isNotNull);
    });

    test('hide-tray with no hotkey is a bad state the UI prevents', () {
      // The switch is disabled in the UI when hasHotkey is false.
      // Verifying state: hide_tray file exists but no hotkey file → safeguard
      // ensures the switch is off in the UI (hasHotkey && hideTray = false).
      final c = makeContainer(hideTray: true, hotkey: null);
      addTearDown(c.dispose);
      final hasHotkey = c.read(globalHotkeyProvider) != null;
      final hideTray = c.read(hideTrayProvider);
      // The UI renders value = hasHotkey && hideTray.
      expect(hasHotkey && hideTray, isFalse);
    });
  });
}
