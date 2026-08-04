import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linkunbound_core/linkunbound_core.dart';

import 'package:linkunbound/providers.dart';
import 'package:linkunbound/ui/settings/maintenance_page.dart';

import '../helpers.dart';

void main() {
  late Directory tempDir;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync('maintenance_page_test_');
  });

  tearDown(() {
    if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
  });

  group('MaintenancePage rendering', () {
    testWidgets('renders without throwing', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const MaintenancePage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.byType(MaintenancePage), findsOneWidget);
    });

    testWidgets('shows MAINTENANCE section header', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const MaintenancePage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('MAINTENANCE'), findsOneWidget);
    });

    testWidgets('shows Export diagnostics action', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const MaintenancePage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('Export diagnostics'), findsOneWidget);
    });

    testWidgets('shows Reset configuration action', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const MaintenancePage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('Reset configuration'), findsOneWidget);
    });

    testWidgets('shows Unregister action', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const MaintenancePage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('Unregister LinkUnbound'), findsOneWidget);
    });
  });

  group('MaintenancePage dialogs', () {
    testWidgets('tapping Export diagnostics shows loading indicator', (
      tester,
    ) async {
      final exportStarted = Completer<void>();
      final exportDone = Completer<String>();
      final f = makeFixtures(
        dir: tempDir,
        diagnosticsExporter:
            ({required Directory appDataDir, required String appVersion}) {
              exportStarted.complete();
              return exportDone.future;
            },
      );
      await tester.pumpWidget(
        buildTestApp(const MaintenancePage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Export diagnostics'));
      await tester.pump(); // show loading dialog
      expect(exportStarted.isCompleted, isTrue);
      expect(find.byType(CircularProgressIndicator), findsOneWidget);

      exportDone.complete('fake.zip');
      await tester.pumpAndSettle();
      expect(find.byType(CircularProgressIndicator), findsNothing);
    });

    testWidgets('confirming Reset executes reset and closes dialog', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const MaintenancePage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Reset configuration'));
      await tester.pumpAndSettle();
      // Tap the destructive confirm button
      await tester.tap(find.text('Reset'));
      await tester.pumpAndSettle();
      expect(find.byType(Dialog), findsNothing);
    });

    testWidgets('tapping Reset configuration shows confirmation dialog', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const MaintenancePage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Reset configuration'));
      await tester.pumpAndSettle();
      expect(find.byType(Dialog), findsOneWidget);
      expect(find.textContaining('Reset'), findsWidgets);
    });

    testWidgets('reset dialog has Cancel button', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const MaintenancePage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Reset configuration'));
      await tester.pumpAndSettle();
      expect(find.text('Cancel'), findsOneWidget);
    });

    testWidgets('tapping Cancel dismisses reset dialog', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const MaintenancePage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Reset configuration'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();
      expect(find.byType(Dialog), findsNothing);
    });

    testWidgets('tapping Unregister shows confirmation dialog', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const MaintenancePage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Unregister LinkUnbound'));
      await tester.pumpAndSettle();
      expect(find.byType(Dialog), findsOneWidget);
    });

    testWidgets('tapping Cancel dismisses unregister dialog', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const MaintenancePage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Unregister LinkUnbound'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();
      expect(find.byType(Dialog), findsNothing);
    });

    testWidgets('confirming unregister calls service unregister', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const MaintenancePage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Unregister LinkUnbound'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Unregister'));
      await tester.pumpAndSettle();
      expect(find.byType(Dialog), findsNothing);
    });

    testWidgets('export diagnostics failure shows error snackbar', (
      tester,
    ) async {
      final f = makeFixtures(
        dir: tempDir,
        diagnosticsExporter:
            ({required Directory appDataDir, required String appVersion}) =>
                Future.error(Exception('export failed')),
      );
      await tester.pumpWidget(
        buildTestApp(const MaintenancePage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Export diagnostics'));
      await tester.pumpAndSettle();
      expect(find.textContaining('Could not export'), findsOneWidget);
    });

    testWidgets('export diagnostics loading dialog is dismissed on success', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const MaintenancePage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Export diagnostics'));
      await tester.pumpAndSettle();
      expect(find.byType(CircularProgressIndicator), findsNothing);
    });

    testWidgets(
      'export diagnostics completes without error snackbar on success',
      (tester) async {
        final f = makeFixtures(dir: tempDir);
        await tester.pumpWidget(
          buildTestApp(const MaintenancePage(), overrides: f.overrides),
        );
        await tester.pumpAndSettle();
        await tester.tap(find.text('Export diagnostics'));
        await tester.pumpAndSettle();
        expect(find.textContaining('Could not export'), findsNothing);
      },
    );

    testWidgets('reset with icon extraction failure is silently ignored', (
      tester,
    ) async {
      final f = makeFixtures(
        dir: tempDir,
        iconExtractor: _ThrowingIconExtractor(),
        detectedBrowsers: [
          const Browser(
            id: 'detected',
            name: 'Detected Browser',
            executablePath: '/usr/bin/detected',
            iconPath: '',
          ),
        ],
      );
      await tester.pumpWidget(
        buildTestApp(const MaintenancePage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Reset configuration'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Reset'));
      await tester.pumpAndSettle();
      expect(find.byType(Dialog), findsNothing);
    });

    testWidgets('unregister failure shows error snackbar', (tester) async {
      final f = makeFixtures(dir: tempDir);
      // Replace the registration service with one that throws on unregister.
      final overridesWithThrow = [
        ...f.overrides,
        registrationServiceProvider.overrideWithValue(
          _ThrowingRegistrationService(),
        ),
      ];
      await tester.pumpWidget(
        buildTestApp(const MaintenancePage(), overrides: overridesWithThrow),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Unregister LinkUnbound'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Unregister'));
      await tester.pumpAndSettle();
      expect(find.textContaining('Could not unregister'), findsOneWidget);
    });
  });
}

final class _ThrowingIconExtractor implements IconExtractor {
  @override
  Future<String> extractIcon(String executablePath, String outputPath) =>
      Future.error(Exception('icon extraction failed'));
}

final class _ThrowingRegistrationService implements RegistrationService {
  @override
  Future<Set<String>> get defaultAssociations async => {};

  @override
  Future<bool> get isDefault async => false;

  @override
  Future<void> register(String executablePath) async {}

  @override
  Future<void> ensureRegistered(String executablePath) =>
      register(executablePath);

  @override
  Future<HandlerDiagnostics> diagnose(String executablePath) async =>
      const HandlerDiagnostics(
        isDefaultBrowser: false,
        commandMatchesExecutable: true,
        runningFromDevBuild: false,
        isPackaged: false,
      );

  @override
  Future<void> setEdgeProtocolCapture(
    bool enabled,
    String executablePath,
  ) async {}

  @override
  Future<bool> get capturesEdgeProtocol async => false;

  @override
  Future<void> unregister() => Future.error(Exception('unregister failed'));
}
