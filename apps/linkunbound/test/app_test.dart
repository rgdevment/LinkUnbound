import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linkunbound_core/linkunbound_core.dart';

import 'package:linkunbound/app.dart';
import 'package:linkunbound/providers.dart';
import 'package:linkunbound/ui/picker/picker_window.dart';
import 'package:linkunbound/ui/settings/settings_window.dart';
import 'package:linkunbound/ui/shared/widgets/title_bar.dart';

import 'helpers.dart';

const _windowChannel = MethodChannel('window_manager');

final class _WindowManagerSpy {
  final List<MethodCall> calls = [];

  List<String> get methods => calls.map((call) => call.method).toList();

  void clear() => calls.clear();

  Future<dynamic> handle(MethodCall call) async {
    calls.add(call);
    switch (call.method) {
      case 'isFullScreen':
      case 'isMaximized':
      case 'isMinimized':
      case 'isVisible':
      case 'isFocused':
        return false;
      default:
        return null;
    }
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  late Directory tempDir;
  late _WindowManagerSpy windowSpy;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync('navigate_app_test_');
    windowSpy = _WindowManagerSpy();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_windowChannel, windowSpy.handle);
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_windowChannel, null);
    if (tempDir.existsSync()) {
      tempDir.deleteSync(recursive: true);
    }
  });

  Future<ProviderContainer> pumpApp(
    WidgetTester tester, {
    UpdateInfo? updateInfo,
    Future<void> Function()? onExit,
  }) async {
    final fixtures = makeFixtures(dir: tempDir, updateInfo: updateInfo);
    final overrides = <Override>[
      ...fixtures.overrides,
      exitAppProvider.overrideWithValue(onExit ?? () async {}),
    ];
    final container = ProviderContainer(overrides: overrides);
    addTearDown(container.dispose);

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const NavigateApp(),
      ),
    );
    await tester.pump();

    return container;
  }

  testWidgets('starts hidden by default', (tester) async {
    await pumpApp(tester);

    expect(find.byType(SettingsWindow), findsNothing);
    expect(find.byType(PickerWindow), findsNothing);
    expect(
      find.byWidgetPredicate(
        (widget) =>
            widget is ColoredBox && widget.color == const Color(0xFF1E1E2E),
      ),
      findsOneWidget,
    );
  });

  testWidgets('showSettings renders the settings window and focuses it', (
    tester,
  ) async {
    final container = await pumpApp(tester);
    windowSpy.clear();

    container.read(appStateProvider.notifier).showSettings();
    await tester.pump();
    await tester.pump();

    expect(find.byType(SettingsWindow), findsOneWidget);
    // Showing and focusing is bootstrap's job now — driving the window from
    // here too raced the serialised transition queue. See bootstrap_test.
  });

  testWidgets('immediate blur after showing picker is ignored', (tester) async {
    final container = await pumpApp(tester);
    final dynamic state = tester.state(find.byType(NavigateApp));

    container.read(appStateProvider.notifier).showPicker('https://example.com');
    await tester.pump();
    await tester.pump();
    // ignore: avoid_dynamic_calls
    state.onWindowBlur();
    await tester.pump();

    expect(find.byType(PickerWindow), findsOneWidget);
  });

  testWidgets('blur hides picker after the grace period', (tester) async {
    final container = await pumpApp(tester);
    final dynamic state = tester.state(find.byType(NavigateApp));

    container.read(appStateProvider.notifier).showPicker('https://example.com');
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 500));
    // ignore: avoid_dynamic_calls
    state.onWindowBlur();
    await tester.pump();

    expect(container.read(appStateProvider).mode, AppMode.hidden);
    expect(find.byType(PickerWindow), findsNothing);
  });

  testWidgets('window close hides the app and resets state', (tester) async {
    final container = await pumpApp(tester);
    final dynamic state = tester.state(find.byType(NavigateApp));

    container.read(appStateProvider.notifier).showSettings();
    await tester.pump();
    await tester.pump();
    windowSpy.clear();

    // ignore: avoid_dynamic_calls
    await state.onWindowClose();
    await tester.pump();

    expect(container.read(appStateProvider).mode, AppMode.hidden);
    expect(windowSpy.methods, contains('hide'));
  });

  testWidgets('settings view shows update banner and supports drag', (
    tester,
  ) async {
    await pumpApp(
      tester,
      updateInfo: const UpdateInfo(
        latestVersion: '2.0.0',
        releaseUrl: 'https://example.com/releases/2.0.0',
      ),
    );

    final container = ProviderScope.containerOf(
      tester.element(find.byType(NavigateApp)),
    );
    container.read(appStateProvider.notifier).showSettings();
    await tester.pump();
    await tester.pump();

    expect(find.text('Version 2.0.0 available'), findsOneWidget);

    windowSpy.clear();
    await tester.drag(find.byType(TitleBar), const Offset(20, 0));
    await tester.pump();
    expect(windowSpy.methods, contains('startDragging'));
  });
}
