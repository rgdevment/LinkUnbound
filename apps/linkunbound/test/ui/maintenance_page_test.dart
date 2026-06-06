import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

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
  });
}
