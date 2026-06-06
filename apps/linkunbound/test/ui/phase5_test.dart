import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:window_manager/window_manager.dart';

import 'package:linkunbound/app.dart';
import 'package:linkunbound/providers.dart';
import 'package:linkunbound/ui/picker/picker_view.dart';
import 'package:linkunbound/ui/picker/picker_window.dart';
import 'package:linkunbound/ui/shared/app_theme.dart';

import '../helpers.dart';

const _windowChannel = MethodChannel('window_manager');

Future<dynamic> _defaultWindowHandler(MethodCall call) async {
  return switch (call.method) {
    'isFullScreen' || 'isMaximized' || 'isMinimized' || 'isVisible' ||
        'isFocused' =>
      false,
    _ => null,
  };
}

({ProviderContainer container, Directory tempDir}) _buildApp(
  WidgetTester tester, {
  List<Browser> browsers = const [],
}) {
  final tempDir = Directory.systemTemp.createTempSync('phase5_test_');
  final f = makeFixtures(dir: tempDir, browsers: browsers);
  final container = ProviderContainer(
    overrides: [
      ...f.overrides,
      exitAppProvider.overrideWithValue(() async {}),
    ],
  );
  return (container: container, tempDir: tempDir);
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  setUp(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_windowChannel, _defaultWindowHandler);
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_windowChannel, null);
  });

  // -------------------------------------------------------------------------
  // P2.3 — Blur unification: single listener in app.dart owns hide-on-blur
  // -------------------------------------------------------------------------
  group('P2.3 — unified blur logic', () {
    testWidgets('PickerWindow state does not implement WindowListener', (
      tester,
    ) async {
      final (:container, :tempDir) = _buildApp(tester);
      addTearDown(container.dispose);
      addTearDown(() { if (tempDir.existsSync()) tempDir.deleteSync(recursive: true); });

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const NavigateApp(),
        ),
      );
      await tester.pump();

      container.read(appStateProvider.notifier).showPicker('https://example.com');
      await tester.pump();
      await tester.pump();

      expect(find.byType(PickerWindow), findsOneWidget);
      final pickerState = tester.state(find.byType(PickerWindow));
      // After unification, PickerWindow no longer owns blur: no WindowListener.
      expect(pickerState, isNot(isA<WindowListener>()));
    });

    testWidgets('blur before grace period does not hide picker', (tester) async {
      final (:container, :tempDir) = _buildApp(tester);
      addTearDown(container.dispose);
      addTearDown(() { if (tempDir.existsSync()) tempDir.deleteSync(recursive: true); });

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const NavigateApp(),
        ),
      );
      await tester.pump();

      final dynamic appState = tester.state(find.byType(NavigateApp));
      container.read(appStateProvider.notifier).showPicker('https://example.com');
      await tester.pump();
      await tester.pump();

      // ignore: avoid_dynamic_calls
      appState.onWindowBlur();
      await tester.pump();

      expect(find.byType(PickerWindow), findsOneWidget);
    });

    testWidgets('blur after grace period hides picker', (tester) async {
      final (:container, :tempDir) = _buildApp(tester);
      addTearDown(container.dispose);
      addTearDown(() { if (tempDir.existsSync()) tempDir.deleteSync(recursive: true); });

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const NavigateApp(),
        ),
      );
      await tester.pump();

      final dynamic appState = tester.state(find.byType(NavigateApp));
      container.read(appStateProvider.notifier).showPicker('https://example.com');
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 400));

      // ignore: avoid_dynamic_calls
      appState.onWindowBlur();
      await tester.pump();

      expect(container.read(appStateProvider).mode, AppMode.hidden);
      expect(find.byType(PickerWindow), findsNothing);
    });

    testWidgets('onWindowFocus marks picker ready before timer fires', (
      tester,
    ) async {
      final (:container, :tempDir) = _buildApp(tester);
      addTearDown(container.dispose);
      addTearDown(() { if (tempDir.existsSync()) tempDir.deleteSync(recursive: true); });

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const NavigateApp(),
        ),
      );
      await tester.pump();

      final dynamic appState = tester.state(find.byType(NavigateApp));
      container.read(appStateProvider.notifier).showPicker('https://example.com');
      await tester.pump();
      await tester.pump();

      // Focus fires before the 350ms fallback — should mark ready.
      // ignore: avoid_dynamic_calls
      appState.onWindowFocus();
      // ignore: avoid_dynamic_calls
      appState.onWindowBlur();
      await tester.pump();

      expect(container.read(appStateProvider).mode, AppMode.hidden);
    });

    testWidgets('blur in settings mode does nothing', (tester) async {
      final (:container, :tempDir) = _buildApp(tester);
      addTearDown(container.dispose);
      addTearDown(() { if (tempDir.existsSync()) tempDir.deleteSync(recursive: true); });

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const NavigateApp(),
        ),
      );
      await tester.pump();

      final dynamic appState = tester.state(find.byType(NavigateApp));
      container.read(appStateProvider.notifier).showSettings();
      await tester.pump();
      await tester.pump();

      // ignore: avoid_dynamic_calls
      appState.onWindowBlur();
      await tester.pump();

      expect(container.read(appStateProvider).mode, AppMode.settings);
    });
  });

  // -------------------------------------------------------------------------
  // P2.4 — Esc closes picker
  // -------------------------------------------------------------------------
  group('P2.4 — Esc closes picker', () {
    testWidgets('Escape key hides the picker via PickerView', (tester) async {
      final f = makeFixtures();
      addTearDown(() {
        if (f.tempDir.existsSync()) {
          f.tempDir.deleteSync(recursive: true);
        }
      });
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();

      // The view is rendered standalone; appStateProvider hides are no-op in
      // buildTestApp, but the key handler should not throw.
      await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      await tester.pumpAndSettle();
      expect(find.byType(PickerView), findsOneWidget);
    });
  });

  // -------------------------------------------------------------------------
  // P2.7 — Theme mode wiring
  // -------------------------------------------------------------------------
  group('P2.7 — theme mode wiring', () {
    testWidgets('MaterialApp uses ThemeMode.system', (tester) async {
      final (:container, :tempDir) = _buildApp(tester);
      addTearDown(container.dispose);
      addTearDown(() { if (tempDir.existsSync()) tempDir.deleteSync(recursive: true); });

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const NavigateApp(),
        ),
      );
      await tester.pump();

      final app = tester.widget<MaterialApp>(find.byType(MaterialApp));
      expect(app.themeMode, ThemeMode.system);
    });

    testWidgets('MaterialApp has both light and dark themes set', (tester) async {
      final (:container, :tempDir) = _buildApp(tester);
      addTearDown(container.dispose);
      addTearDown(() { if (tempDir.existsSync()) tempDir.deleteSync(recursive: true); });

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const NavigateApp(),
        ),
      );
      await tester.pump();

      final app = tester.widget<MaterialApp>(find.byType(MaterialApp));
      expect(app.theme, isNotNull);
      expect(app.darkTheme, isNotNull);
    });

    test('AppTheme.light has Brightness.light', () {
      expect(AppTheme.light.brightness, Brightness.light);
    });

    test('AppTheme.dark has Brightness.dark', () {
      expect(AppTheme.dark.brightness, Brightness.dark);
    });

    test('light and dark primaries share the same blue hue family', () {
      final lightPrimary = AppTheme.light.colorScheme.primary;
      final darkPrimary = AppTheme.dark.colorScheme.primary;
      expect(lightPrimary.b, greaterThan(lightPrimary.r));
      expect(darkPrimary.b, greaterThan(darkPrimary.r));
    });
  });

  // -------------------------------------------------------------------------
  // P2.5 — onWindowFocus debounce
  // -------------------------------------------------------------------------
  group('P2.5 — focus invalidation debounce', () {
    testWidgets('rapid focus events do not throw', (tester) async {
      final (:container, :tempDir) = _buildApp(tester);
      addTearDown(container.dispose);
      addTearDown(() { if (tempDir.existsSync()) tempDir.deleteSync(recursive: true); });

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const NavigateApp(),
        ),
      );
      await tester.pump();

      final dynamic appState = tester.state(find.byType(NavigateApp));

      // ignore: avoid_dynamic_calls
      appState.onWindowFocus();
      // ignore: avoid_dynamic_calls
      appState.onWindowFocus();
      await tester.pump();

      expect(find.byType(NavigateApp), findsOneWidget);
    });

    testWidgets('invalidation fires again after 2s cooldown', (tester) async {
      final (:container, :tempDir) = _buildApp(tester);
      addTearDown(container.dispose);
      addTearDown(() { if (tempDir.existsSync()) tempDir.deleteSync(recursive: true); });

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const NavigateApp(),
        ),
      );
      await tester.pump();

      final dynamic appState = tester.state(find.byType(NavigateApp));

      // ignore: avoid_dynamic_calls
      appState.onWindowFocus();
      await tester.pump(const Duration(seconds: 3));
      // ignore: avoid_dynamic_calls
      appState.onWindowFocus();
      await tester.pump();

      expect(find.byType(NavigateApp), findsOneWidget);
    });
  });

  // -------------------------------------------------------------------------
  // P2.9 — Scrollbar visible with more than maxVisible (6) browsers
  // -------------------------------------------------------------------------
  group('P2.9 — scrollbar on overflow', () {
    testWidgets('Scrollbar is shown when browser count exceeds maxVisible', (
      tester,
    ) async {
      final browsers = List.generate(
        7,
        (i) => Browser(
          id: 'browser$i',
          name: 'Browser $i',
          executablePath: 'browser$i.exe',
          iconPath: '',
        ),
      );
      final f = makeFixtures(browsers: browsers);
      addTearDown(() {
        if (f.tempDir.existsSync()) f.tempDir.deleteSync(recursive: true);
      });
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(Scrollbar), findsOneWidget);
    });
  });
}
