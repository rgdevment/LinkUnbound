import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:linkunbound/platform/hotkey_service.dart';
import 'package:linkunbound/ui/settings/general_page.dart';

import '../helpers.dart';

void main() {
  late Directory tempDir;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync('general_p03_test_');
  });

  tearDown(() async {
    for (var i = 0; i < 5; i++) {
      try {
        tempDir.deleteSync(recursive: true);
        break;
      } on FileSystemException {
        await Future<void>.delayed(const Duration(milliseconds: 100));
      }
    }
  });

  group('GeneralPage — accessibility section', () {
    testWidgets('shows ACCESSIBILITY section header', (tester) async {
      tester.view.physicalSize = const Size(800, 2400);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('ACCESSIBILITY'), findsOneWidget);
    });

    testWidgets('shows global hotkey label', (tester) async {
      tester.view.physicalSize = const Size(800, 2400);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('Global shortcut to open settings'), findsOneWidget);
    });

    testWidgets('shows hide tray label', (tester) async {
      tester.view.physicalSize = const Size(800, 2400);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('Hide tray / menu bar icon'), findsOneWidget);
    });

    testWidgets('hide tray switch is disabled when no hotkey configured', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(800, 1600);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();

      // The hide-tray switch is the one with onChanged == null (disabled).
      // There are multiple Switch widgets: startup + hide-tray.
      final switches = tester.widgetList<Switch>(find.byType(Switch)).toList();
      // The last switch in the page is the hide-tray one (added after startup).
      final hideTraySwitch = switches.last;
      expect(hideTraySwitch.onChanged, isNull,
          reason: 'hide-tray switch must be disabled without a hotkey');
    });

    testWidgets(
      'hide tray switch is enabled when hotkey is configured',
      (tester) async {
        tester.view.physicalSize = const Size(800, 1600);
        tester.view.devicePixelRatio = 1.0;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);

        // Write hotkey file before makeFixtures reads it via the provider.
        File('${tempDir.path}/global_hotkey')
            .writeAsStringSync(HotkeyPreset.defaults.first.serialized);

        final f = makeFixtures(dir: tempDir);
        await tester.pumpWidget(
          buildTestApp(const GeneralPage(), overrides: f.overrides),
        );
        await tester.pumpAndSettle();

        final switches = tester.widgetList<Switch>(find.byType(Switch)).toList();
        final hideTraySwitch = switches.last;
        expect(hideTraySwitch.onChanged, isNotNull,
            reason: 'hide-tray switch must be enabled when hotkey is set');
      },
    );

    testWidgets('hotkey dropdown shows None option', (tester) async {
      tester.view.physicalSize = const Size(800, 2400);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      // The dropdown button for hotkeys shows the current value.
      expect(find.text('None (disabled)'), findsOneWidget);
    });
  });
}
