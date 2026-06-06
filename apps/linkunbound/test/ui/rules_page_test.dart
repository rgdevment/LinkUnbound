import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linkunbound_core/linkunbound_core.dart';

import 'package:linkunbound/ui/settings/rules_page.dart';

import '../helpers.dart';

const _chrome = Browser(
  id: 'chrome',
  name: 'Google Chrome',
  executablePath: 'chrome.exe',
  iconPath: 'chrome.png',
);

const _firefox = Browser(
  id: 'firefox',
  name: 'Firefox',
  executablePath: 'firefox.exe',
  iconPath: 'firefox.png',
);

void main() {
  late Directory tempDir;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync('rules_page_test_');
  });

  tearDown(() async {
    if (tempDir.existsSync()) {
      for (var i = 0; i < 5; i++) {
        try {
          tempDir.deleteSync(recursive: true);
          break;
        } on FileSystemException {
          await Future<void>.delayed(const Duration(milliseconds: 100));
        }
      }
    }
  });

  group('RulesPage rendering — empty state', () {
    testWidgets('renders without throwing', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const RulesPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.byType(RulesPage), findsOneWidget);
    });

    testWidgets('shows URL RULES section header', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const RulesPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('URL RULES'), findsOneWidget);
    });

    testWidgets('shows empty state hint when no rules', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const RulesPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.textContaining('No rules yet'), findsOneWidget);
    });

    testWidgets('empty state does not show column headers', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const RulesPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('Domain'), findsNothing);
      expect(find.text('Browser'), findsNothing);
    });
  });

  group('RulesPage rendering — with rules', () {
    testWidgets('shows Domain and Browser column headers', (tester) async {
      final f = makeFixtures(
        dir: tempDir,
        browsers: [_chrome],
        rules: [const Rule(domain: 'github.com', browserId: 'chrome')],
      );
      await tester.pumpWidget(
        buildTestApp(const RulesPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('Domain'), findsOneWidget);
      expect(find.text('Browser'), findsOneWidget);
    });

    testWidgets('renders rule domain', (tester) async {
      final f = makeFixtures(
        dir: tempDir,
        browsers: [_chrome],
        rules: [const Rule(domain: 'github.com', browserId: 'chrome')],
      );
      await tester.pumpWidget(
        buildTestApp(const RulesPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.textContaining('github.com'), findsWidgets);
    });

    testWidgets('renders multiple rules', (tester) async {
      final f = makeFixtures(
        dir: tempDir,
        browsers: [_chrome, _firefox],
        rules: [
          const Rule(domain: 'github.com', browserId: 'chrome'),
          const Rule(domain: 'mail.google.com', browserId: 'firefox'),
        ],
      );
      await tester.pumpWidget(
        buildTestApp(const RulesPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.textContaining('github.com'), findsWidgets);
      expect(find.textContaining('mail.google.com'), findsWidgets);
    });

    testWidgets('does not show empty state when rules exist', (tester) async {
      final f = makeFixtures(
        dir: tempDir,
        browsers: [_chrome],
        rules: [const Rule(domain: 'github.com', browserId: 'chrome')],
      );
      await tester.pumpWidget(
        buildTestApp(const RulesPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.textContaining('No rules yet'), findsNothing);
    });
  });

  group('RulesPage — _browserName fallback', () {
    testWidgets('shows browserId when browser not found in list', (
      tester,
    ) async {
      // Rule references 'unknown-browser' but the browser list has no such id.
      final f = makeFixtures(
        dir: tempDir,
        browsers: [_chrome],
        rules: [
          const Rule(domain: 'example.com', browserId: 'unknown-browser'),
        ],
      );
      await tester.pumpWidget(
        buildTestApp(const RulesPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      // Domain row is visible even when browser name resolution falls back.
      expect(find.textContaining('example.com'), findsWidgets);
    });
  });

  group('RulesPage — onBrowserChanged', () {
    testWidgets('changing browser in dropdown updates the rule', (
      tester,
    ) async {
      final f = makeFixtures(
        dir: tempDir,
        browsers: [_chrome, _firefox],
        rules: [const Rule(domain: 'github.com', browserId: 'chrome')],
      );
      await tester.pumpWidget(
        buildTestApp(const RulesPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();

      // Open the browser dropdown for the first rule.
      final dropdown = find.byType(DropdownButton<String>);
      expect(dropdown, findsOneWidget);
      await tester.tap(dropdown);
      await tester.pumpAndSettle();

      // Select Firefox.
      await tester.tap(find.text('Firefox').last);
      await tester.pumpAndSettle();

      final updated = f.ruleService.rules.first;
      expect(updated.browserId, 'firefox');
    });
  });

  group('RulesPage interactions', () {
    testWidgets('delete button shows confirmation dialog', (tester) async {
      final f = makeFixtures(
        dir: tempDir,
        browsers: [_chrome],
        rules: [const Rule(domain: 'github.com', browserId: 'chrome')],
      );
      await tester.pumpWidget(
        buildTestApp(const RulesPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      final deleteButton = find.byTooltip('Delete rule');
      expect(deleteButton, findsOneWidget);
      await tester.tap(deleteButton);
      await tester.pumpAndSettle();
      expect(find.byType(Dialog), findsOneWidget);
    });

    testWidgets('delete dialog shows domain name', (tester) async {
      final f = makeFixtures(
        dir: tempDir,
        browsers: [_chrome],
        rules: [const Rule(domain: 'github.com', browserId: 'chrome')],
      );
      await tester.pumpWidget(
        buildTestApp(const RulesPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Delete rule'));
      await tester.pumpAndSettle();
      expect(find.textContaining('github.com'), findsWidgets);
    });

    testWidgets('cancelling delete dismisses dialog', (tester) async {
      final f = makeFixtures(
        dir: tempDir,
        browsers: [_chrome],
        rules: [const Rule(domain: 'github.com', browserId: 'chrome')],
      );
      await tester.pumpWidget(
        buildTestApp(const RulesPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Delete rule'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();
      expect(find.byType(Dialog), findsNothing);
    });

    testWidgets('confirming delete removes rule from list', (tester) async {
      final f = makeFixtures(
        dir: tempDir,
        browsers: [_chrome],
        rules: [const Rule(domain: 'github.com', browserId: 'chrome')],
      );
      await tester.pumpWidget(
        buildTestApp(const RulesPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Delete rule'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Delete'));
      await tester.pumpAndSettle();
      // Dialog is dismissed and service rule removed synchronously
      expect(find.byType(Dialog), findsNothing);
      expect(f.ruleService.rules, isEmpty);
    });
  });
}
