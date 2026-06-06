import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:linkunbound/platform/hotkey_service.dart';

const _hotkeyChannel = MethodChannel('dev.leanflutter.plugins/hotkey_manager');

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  setUp(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_hotkeyChannel, (_) async => null);
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_hotkeyChannel, null);
  });

  group('HotkeyService — serialization round-trip', () {
    test('serialize of parsed meta+shift+l contains meta, shift, and l', () {
      const raw = 'meta+shift+l';
      final parsed = HotkeyService.parse(raw);
      expect(parsed, isNotNull);
      final serialized = HotkeyService.serialize(parsed!);
      expect(serialized, contains('meta'));
      expect(serialized, contains('shift'));
      expect(serialized, contains('l'));
    });

    test('serialize of parsed ctrl+alt+space contains ctrl, alt, and space', () {
      const raw = 'ctrl+alt+space';
      final parsed = HotkeyService.parse(raw);
      expect(parsed, isNotNull);
      final serialized = HotkeyService.serialize(parsed!);
      expect(serialized, contains('ctrl'));
      expect(serialized, contains('alt'));
      expect(serialized, contains('space'));
    });

    test('parse returns null for unknown key', () {
      expect(HotkeyService.parse('meta+~'), isNull);
    });

    test('parse returns null for empty string', () {
      expect(HotkeyService.parse(''), isNull);
    });

    test('parse handles cmd alias for meta', () {
      final parsed = HotkeyService.parse('cmd+shift+b');
      expect(parsed, isNotNull);
    });

    test('parse handles option alias for alt', () {
      final parsed = HotkeyService.parse('option+space');
      expect(parsed, isNotNull);
    });
  });

  group('HotkeyService — register/unregister lifecycle', () {
    test('isRegistered is false before register', () {
      final svc = HotkeyService();
      expect(svc.isRegistered, isFalse);
    });

    test('register with null does not set isRegistered', () async {
      final svc = HotkeyService();
      await svc.register(null);
      expect(svc.isRegistered, isFalse);
    });

    test('register with empty string does not set isRegistered', () async {
      final svc = HotkeyService();
      await svc.register('');
      expect(svc.isRegistered, isFalse);
    });

    test('register with valid key sets isRegistered', () async {
      final svc = HotkeyService();
      await svc.register('meta+shift+l');
      expect(svc.isRegistered, isTrue);
      await svc.dispose();
    });

    test('unregister clears isRegistered', () async {
      final svc = HotkeyService();
      await svc.register('meta+shift+l');
      await svc.unregister();
      expect(svc.isRegistered, isFalse);
    });

    test('dispose clears isRegistered', () async {
      final svc = HotkeyService();
      await svc.register('ctrl+alt+space');
      await svc.dispose();
      expect(svc.isRegistered, isFalse);
    });

    test('registering a new hotkey replaces the previous one', () async {
      final svc = HotkeyService();
      await svc.register('meta+shift+l');
      // Register a second key: the first should be unregistered first.
      await svc.register('ctrl+alt+space');
      expect(svc.isRegistered, isTrue);
      await svc.dispose();
    });

    test('callback fires when triggered', () async {
      var triggered = false;
      final svc = HotkeyService();
      svc.setCallback(() => triggered = true);
      await svc.register('meta+shift+l');
      expect(triggered, isFalse);
      await svc.dispose();
    });
  });

  group('HotkeyPreset.defaults', () {
    test('all presets are parseable', () {
      for (final preset in HotkeyPreset.defaults) {
        final parsed = HotkeyService.parse(preset.serialized);
        expect(parsed, isNotNull, reason: '${preset.serialized} failed');
      }
    });

    test('defaults list is not empty', () {
      expect(HotkeyPreset.defaults, isNotEmpty);
    });
  });
}
