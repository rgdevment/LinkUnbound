import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:linkunbound/platform/macos/mac_window_channel.dart';

const _channel = MethodChannel('linkunbound/window');

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  late List<String> methodCalls;

  setUp(() {
    methodCalls = [];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_channel, (call) async {
          methodCalls.add(call.method);
          return null;
        });
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_channel, null);
  });

  test('setPickerMode invokes native channel', () async {
    final channel = MacWindowChannel();

    await channel.setPickerMode();

    expect(methodCalls, ['setPickerMode']);
  });

  test('setSettingsMode invokes native channel', () async {
    final channel = MacWindowChannel();

    await channel.setSettingsMode();

    expect(methodCalls, ['setSettingsMode']);
  });

  test('activate invokes native channel', () async {
    final channel = MacWindowChannel();

    await channel.activate();

    expect(methodCalls, ['activate']);
  });

  test('setRegular invokes native channel', () async {
    final channel = MacWindowChannel();

    await channel.setRegular();

    expect(methodCalls, ['setRegular']);
  });

  test('setAccessory invokes native channel', () async {
    final channel = MacWindowChannel();

    await channel.setAccessory();

    expect(methodCalls, ['setAccessory']);
  });
}
