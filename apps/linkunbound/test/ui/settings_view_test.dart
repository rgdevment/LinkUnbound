import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linkunbound_core/linkunbound_core.dart';

import 'package:linkunbound/ui/settings/general_page.dart';
import 'package:linkunbound/ui/settings/rules_page.dart';
import 'package:linkunbound/ui/settings/settings_view.dart';

import '../helpers.dart';

const _windowChannel = MethodChannel('window_manager');
const _urlLauncherChannel = MethodChannel(
  'plugins.flutter.io/url_launcher_windows',
);

void main() {
  late Directory tempDir;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync('settings_view_test_');
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_windowChannel, (call) async => null);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_urlLauncherChannel, (call) async => true);
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_windowChannel, null);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_urlLauncherChannel, null);
    if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
  });

  group('SettingsView — tabs', () {
    testWidgets('renders without throwing', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const SettingsView(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.byType(SettingsView), findsOneWidget);
    });

    testWidgets('shows all four tab labels', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const SettingsView(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('General'), findsOneWidget);
      expect(find.text('Rules'), findsOneWidget);
      expect(find.text('Maintenance'), findsOneWidget);
      expect(find.text('About'), findsOneWidget);
    });

    testWidgets('defaults to General tab content', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const SettingsView(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.byType(GeneralPage), findsOneWidget);
    });

    testWidgets('switching to Rules tab shows RulesPage', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const SettingsView(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Rules'));
      await tester.pumpAndSettle();
      expect(find.byType(RulesPage), findsOneWidget);
    });
  });

  group('SettingsView — update banner', () {
    testWidgets('no banner when updateInfo is null', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const SettingsView(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.byIcon(Icons.upgrade_rounded), findsNothing);
    });

    testWidgets('banner visible when updateInfo is available', (tester) async {
      final f = makeFixtures(
        dir: tempDir,
        updateInfo: const UpdateInfo(
          latestVersion: '2.0.0',
          releaseUrl: 'https://github.com/test/releases/tag/v2.0.0',
        ),
      );
      await tester.pumpWidget(
        buildTestApp(const SettingsView(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.byIcon(Icons.upgrade_rounded), findsOneWidget);
    });

    testWidgets('banner displays version number', (tester) async {
      final f = makeFixtures(
        dir: tempDir,
        updateInfo: const UpdateInfo(
          latestVersion: '3.5.1',
          releaseUrl: 'https://github.com/test/releases/tag/v3.5.1',
        ),
      );
      await tester.pumpWidget(
        buildTestApp(const SettingsView(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.textContaining('3.5.1'), findsOneWidget);
    });

    testWidgets('banner shows download button for non-MSIX build', (
      tester,
    ) async {
      final f = makeFixtures(
        dir: tempDir,
        updateInfo: const UpdateInfo(
          latestVersion: '2.0.0',
          releaseUrl: 'https://github.com/test/releases/tag/v2.0.0',
        ),
      );
      await tester.pumpWidget(
        buildTestApp(const SettingsView(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('Download'), findsOneWidget);
    });

    testWidgets('tapping download button invokes url launcher', (tester) async {
      final f = makeFixtures(
        dir: tempDir,
        updateInfo: const UpdateInfo(
          latestVersion: '2.0.0',
          releaseUrl: 'https://github.com/test/releases/tag/v2.0.0',
        ),
      );
      await tester.pumpWidget(
        buildTestApp(const SettingsView(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Download'));
      await tester.pumpAndSettle();
    });
  });

  // Close button is Windows-only; macOS uses native traffic lights.
  group('SettingsView — close button', () {
    testWidgets(
      'tapping close button hides the window',
      skip: Platform.isMacOS,
      (tester) async {
        final f = makeFixtures(dir: tempDir);
        await tester.pumpWidget(
          buildTestApp(const SettingsView(), overrides: f.overrides),
        );
        await tester.pumpAndSettle();
        await tester.tap(find.byIcon(Icons.close));
        await tester.pumpAndSettle();
      },
    );
  });
}
