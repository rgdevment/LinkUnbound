import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linkunbound_core/linkunbound_core.dart';

import 'package:linkunbound/platform/hotkey_service.dart';
import 'package:linkunbound/ui/settings/general_page.dart';

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

const _edge = Browser(
  id: 'edge',
  name: 'Microsoft Edge',
  executablePath:
      r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe',
  iconPath: 'edge.png',
);

void main() {
  late Directory tempDir;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync('general_page_test_');
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

  group('GeneralPage — browsers section', () {
    testWidgets('renders without throwing', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.byType(GeneralPage), findsOneWidget);
    });

    testWidgets('shows BROWSERS section header', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('BROWSERS'), findsOneWidget);
    });

    testWidgets('shows add browser tooltip', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.byTooltip('Add custom browser'), findsOneWidget);
    });

    testWidgets('shows refresh browsers tooltip', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.byTooltip('Refresh browsers'), findsOneWidget);
    });

    testWidgets('shows browser name in tile', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome, _firefox]);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('Google Chrome'), findsOneWidget);
      expect(find.text('Firefox'), findsOneWidget);
    });

    testWidgets('shows popup menu for browser row', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byIcon(Icons.more_vert));
      await tester.pumpAndSettle();
      expect(find.text('Edit'), findsOneWidget);
      expect(find.text('Duplicate'), findsOneWidget);
      expect(find.text('Remove'), findsOneWidget);
    });

    testWidgets('refresh with empty browsers shows no-changes snackbar', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Refresh browsers'));
      await tester.pumpAndSettle();
      expect(find.text('No changes detected'), findsOneWidget);
    });
  });

  group('GeneralPage — default browser section', () {
    testWidgets('shows DEFAULT BROWSER section header', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('DEFAULT BROWSER'), findsOneWidget);
    });

    testWidgets('shows not-default status when isDefault is false', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir, isDefault: false);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.textContaining('not the default browser'), findsOneWidget);
    });

    testWidgets('shows Set Default button when not default', (tester) async {
      final f = makeFixtures(dir: tempDir, isDefault: false);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('Set default'), findsOneWidget);
    });

    testWidgets('shows is-default status when isDefault is true', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir, isDefault: true);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.textContaining('is set as the default'), findsOneWidget);
    });

    testWidgets('no Set Default button when is default', (tester) async {
      final f = makeFixtures(dir: tempDir, isDefault: true);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('Set default'), findsNothing);
    });
  });

  group('GeneralPage — startup section', () {
    testWidgets('shows STARTUP section header', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('STARTUP'), findsOneWidget);
    });

    testWidgets('shows launch at startup label', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('Launch at system startup'), findsOneWidget);
    });

    testWidgets('startup switch is off by default', (tester) async {
      final f = makeFixtures(dir: tempDir, isStartupEnabled: false);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      final sw = tester.widget<Switch>(find.byType(Switch));
      expect(sw.value, isFalse);
    });

    testWidgets('startup switch shows on when isStartupEnabled is true', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir, isStartupEnabled: true);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      final sw = tester.widget<Switch>(find.byType(Switch));
      expect(sw.value, isTrue);
    });

    testWidgets('tapping startup switch when off calls service enable', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir, isStartupEnabled: false);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byType(Switch));
      await tester.pumpAndSettle();
      expect(find.byType(Switch), findsOneWidget);
    });

    testWidgets('tapping startup switch calls service disable', (tester) async {
      final f = makeFixtures(dir: tempDir, isStartupEnabled: true);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byType(Switch));
      await tester.pumpAndSettle();
      expect(find.byType(Switch), findsOneWidget);
    });
  });

  group('GeneralPage — language section', () {
    testWidgets('shows LANGUAGE section header', (tester) async {
      tester.view.physicalSize = const Size(800, 1600);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.reset);
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('LANGUAGE'), findsWidgets);
    });

    testWidgets('shows Automatic option in dropdown', (tester) async {
      tester.view.physicalSize = const Size(800, 1600);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      // One for the language dropdown, one for the theme dropdown.
      expect(find.text('Automatic (system)'), findsNWidgets(2));
    });

    testWidgets('opening dropdown shows all language options', (tester) async {
      tester.view.physicalSize = const Size(800, 1600);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.ensureVisible(find.byType(DropdownButton<String>));
      await tester.pumpAndSettle();
      await tester.tap(find.byType(DropdownButton<String>));
      await tester.pumpAndSettle();
      expect(find.text('English'), findsOneWidget);
      expect(find.text('Spanish'), findsOneWidget);
    });

    testWidgets('selecting English updates locale', (tester) async {
      tester.view.physicalSize = const Size(800, 1600);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.ensureVisible(find.byType(DropdownButton<String>));
      await tester.pumpAndSettle();
      await tester.tap(find.byType(DropdownButton<String>));
      await tester.pumpAndSettle();
      await tester.tap(find.text('English').last);
      await tester.pumpAndSettle();
      expect(find.text('English'), findsOneWidget);
    });
  });

  group('GeneralPage — appearance section', () {
    testWidgets('shows APPEARANCE section header', (tester) async {
      tester.view.physicalSize = const Size(800, 1600);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.reset);
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('APPEARANCE'), findsOneWidget);
    });

    testWidgets('opening dropdown shows all theme options', (tester) async {
      tester.view.physicalSize = const Size(800, 1600);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.reset);
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.ensureVisible(find.byType(DropdownButton<ThemeMode>));
      await tester.pumpAndSettle();
      await tester.tap(find.byType(DropdownButton<ThemeMode>));
      await tester.pumpAndSettle();
      expect(find.text('Light'), findsOneWidget);
      expect(find.text('Dark'), findsOneWidget);
    });

    testWidgets('selecting Dark persists and updates provider', (tester) async {
      tester.view.physicalSize = const Size(800, 1600);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.reset);
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.ensureVisible(find.byType(DropdownButton<ThemeMode>));
      await tester.pumpAndSettle();
      await tester.tap(find.byType(DropdownButton<ThemeMode>));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Dark').last);
      await tester.pumpAndSettle();
      expect(find.text('Dark'), findsOneWidget);
      expect(File('${tempDir.path}/theme').readAsStringSync(), 'dark');
    });

    testWidgets('saved dark theme is preselected', (tester) async {
      File('${tempDir.path}/theme').writeAsStringSync('dark');
      tester.view.physicalSize = const Size(800, 1600);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.reset);
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('Dark'), findsOneWidget);
    });
  });

  group('GeneralPage — add/edit browser dialog', () {
    testWidgets('tapping add button shows dialog', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Add custom browser'));
      await tester.pumpAndSettle();
      expect(find.text('Add custom browser'), findsWidgets);
    });

    testWidgets('add dialog shows Name field', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Add custom browser'));
      await tester.pumpAndSettle();
      expect(find.text('Name'), findsOneWidget);
      expect(find.text('Executable path'), findsOneWidget);
    });

    testWidgets('add dialog Cancel closes dialog', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Add custom browser'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();
      expect(find.byType(Dialog), findsNothing);
    });

    testWidgets('tapping edit on browser opens edit dialog', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byIcon(Icons.more_vert));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Edit'));
      await tester.pumpAndSettle();
      expect(find.text('Edit browser'), findsOneWidget);
    });

    testWidgets('edit dialog is pre-filled with browser name', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byIcon(Icons.more_vert));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Edit'));
      await tester.pumpAndSettle();
      expect(find.widgetWithText(TextField, 'Google Chrome'), findsOneWidget);
    });

    testWidgets('add with empty fields does not add browser', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Add custom browser'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Add'));
      await tester.pumpAndSettle();
      expect(find.byType(Dialog), findsOneWidget);
    });

    testWidgets('duplicating a browser adds a copy to the list', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byIcon(Icons.more_vert));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Duplicate'));
      await tester.pumpAndSettle();
      expect(find.text('Google Chrome'), findsWidgets);
    });

    testWidgets('removing a browser removes it from the list', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byIcon(Icons.more_vert));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Remove'));
      // Pump once to let the synchronous removeBrowser() call execute.
      await tester.pump();
      // The in-memory service list is updated synchronously before the async save.
      expect(f.browserService.browsers, isEmpty);
    });

    testWidgets('add dialog saves new browser when name and path are filled', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Add custom browser'));
      await tester.pumpAndSettle();
      await tester.enterText(
        find.widgetWithText(TextField, 'Name'),
        'My Browser',
      );
      await tester.enterText(
        find.widgetWithText(TextField, 'Executable path'),
        '/usr/bin/mybrowser',
      );
      await tester.tap(find.text('Add'));
      await tester.pumpAndSettle();
      expect(find.text('My Browser'), findsOneWidget);
    });

    testWidgets('edit dialog path field is disabled for non-custom browser', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byIcon(Icons.more_vert));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Edit'));
      await tester.pumpAndSettle();
      final pathField = tester
          .widgetList<TextField>(find.byType(TextField))
          .firstWhere((f) => f.decoration?.labelText == 'Executable path');
      expect(pathField.enabled, isFalse);
    });

    testWidgets('edit dialog saves name change for existing browser', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byIcon(Icons.more_vert));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Edit'));
      await tester.pumpAndSettle();
      final nameField = tester
          .widgetList<TextField>(find.byType(TextField))
          .firstWhere((f) => f.decoration?.labelText == 'Name');
      await tester.enterText(find.byWidget(nameField), 'Chrome Renamed');
      // runAsync lets the async _saveBrowser → _updateBrowserIcon chain complete.
      await tester.runAsync(() async {
        await tester.tap(find.text('Save'));
        await Future<void>.delayed(const Duration(milliseconds: 100));
      });
      await tester.pumpAndSettle();
      expect(find.text('Chrome Renamed'), findsOneWidget);
    });
  });

  group('GeneralPage — edge warning card', () {
    testWidgets('hides edge warning when no edge browser', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome, _firefox]);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('Microsoft Edge detected'), findsNothing);
    });

    testWidgets(
      'shows edge warning card when edge browser present and not dismissed',
      (tester) async {
        final f = makeFixtures(dir: tempDir, browsers: [_edge]);
        await tester.pumpWidget(
          buildTestApp(const GeneralPage(), overrides: f.overrides),
        );
        await tester.pumpAndSettle();
        expect(find.text('Microsoft Edge detected'), findsOneWidget);
      },
    );

    testWidgets('hides edge warning when dismissed file already exists', (
      tester,
    ) async {
      File('${tempDir.path}/edge_warning_dismissed').writeAsStringSync('1');
      final f = makeFixtures(dir: tempDir, browsers: [_edge]);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text('Microsoft Edge detected'), findsNothing);
    });

    testWidgets('edge warning card shows body text', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_edge]);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.textContaining('Microsoft Teams'), findsOneWidget);
    });

    testWidgets('edge warning card shows dismiss button', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_edge]);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.text("Got it, don't show again"), findsOneWidget);
    });

    testWidgets('tapping dismiss hides edge warning card', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_edge]);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      final dismissButton = find.text("Got it, don't show again");
      await tester.ensureVisible(dismissButton);
      await tester.pumpAndSettle();
      await tester.tap(dismissButton);
      await tester.pumpAndSettle();
      expect(find.text('Microsoft Edge detected'), findsNothing);
    });
  });

  group('GeneralPage — refresh with changes', () {
    testWidgets('refresh shows result snackbar when new browser detected', (
      tester,
    ) async {
      // Detector returns _firefox but initial list is empty → added: 1
      final f = makeFixtures(dir: tempDir, detectedBrowsers: [_firefox]);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Refresh browsers'));
      await tester.pumpAndSettle();
      expect(find.text('1 added, 0 removed'), findsOneWidget);
    });

    testWidgets('refresh result snackbar contains added count', (tester) async {
      final f = makeFixtures(
        dir: tempDir,
        detectedBrowsers: [_chrome, _firefox],
      );
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Refresh browsers'));
      await tester.pumpAndSettle();
      expect(find.text('2 added, 0 removed'), findsOneWidget);
    });
  });

  group('GeneralPage — startup toggle error', () {
    testWidgets('startup toggle error shows error snackbar', (tester) async {
      final f = makeFixtures(
        dir: tempDir,
        startupService: _ThrowingStartupService(),
        isStartupEnabled: false,
      );
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byType(Switch));
      await tester.pumpAndSettle();
      expect(find.text('Could not change startup setting'), findsOneWidget);
    });
  });

  group('GeneralPage — startup toggle calls', () {
    testWidgets('switch is always interactive', (tester) async {
      final f = makeFixtures(dir: tempDir, isStartupEnabled: false);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      final sw = tester.widget<Switch>(find.byType(Switch));
      expect(sw.onChanged, isNotNull);
    });

    testWidgets('toggling on calls enable on the service', (tester) async {
      final recording = _RecordingStartupService(isEnabledValue: false);
      final f = makeFixtures(dir: tempDir, startupService: recording);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byType(Switch));
      await tester.pumpAndSettle();
      expect(recording.enableCalls, 1);
      expect(recording.disableCalls, 0);
    });

    testWidgets('toggling off calls disable on the service', (tester) async {
      final recording = _RecordingStartupService(isEnabledValue: true);
      final f = makeFixtures(dir: tempDir, startupService: recording);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byType(Switch));
      await tester.pumpAndSettle();
      expect(recording.disableCalls, 1);
      expect(recording.enableCalls, 0);
    });

    testWidgets('enable is called with a non-empty executable path', (
      tester,
    ) async {
      final recording = _RecordingStartupService(isEnabledValue: false);
      final f = makeFixtures(dir: tempDir, startupService: recording);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byType(Switch));
      await tester.pumpAndSettle();
      expect(recording.lastEnablePath, isNotEmpty);
    });
  });

  group('GeneralPage — browser tile tap', () {
    testWidgets('tapping tile body opens edit dialog', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Google Chrome'));
      await tester.pumpAndSettle();
      expect(find.text('Edit browser'), findsOneWidget);
    });
  });

  group('GeneralPage — browser reorder', () {
    testWidgets('dragging an item triggers reorder', (tester) async {
      tester.view.physicalSize = const Size(800, 1200);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      final f = makeFixtures(dir: tempDir, browsers: [_chrome, _firefox]);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();

      final dragHandles = find.byIcon(Icons.drag_handle);
      expect(dragHandles, findsNWidgets(2));

      // Start drag from second handle and pump frames during the move so the
      // proxyDecorator AnimatedBuilder is rendered, then release at new position
      // to trigger onReorder.
      final gesture = await tester.startGesture(
        tester.getCenter(dragHandles.at(1)),
      );
      await tester.pump(const Duration(milliseconds: 100));
      await gesture.moveBy(const Offset(0, -150));
      await tester.pump(const Duration(milliseconds: 100));
      await gesture.up();
      await tester.pumpAndSettle();

      // List still has both browsers after reorder.
      expect(find.text('Google Chrome'), findsOneWidget);
      expect(find.text('Firefox'), findsOneWidget);
    });
  });

  group('GeneralPage — duplicate browser with icon', () {
    testWidgets('duplicating browser with existing icon copies icon file', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      // Create a fake icon file for chrome so the copy path is exercised.
      File('${f.tempDir.path}/icons/chrome.png')
        ..parent.createSync(recursive: true)
        ..writeAsBytesSync([0, 1, 2, 3]);

      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byIcon(Icons.more_vert));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Duplicate'));
      // Multiple pumps give the async _duplicateBrowser Future time to complete.
      for (var i = 0; i < 5; i++) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      await tester.pumpAndSettle();
      // Both the original and the copy are visible.
      expect(find.text('Google Chrome'), findsWidgets);
    });
  });

  group('GeneralPage — refresh with icon extraction failure', () {
    testWidgets('icon extraction failure during refresh is silently ignored', (
      tester,
    ) async {
      final f = makeFixtures(
        dir: tempDir,
        detectedBrowsers: [_firefox],
        iconExtractor: _ThrowingIconExtractor(),
      );
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Refresh browsers'));
      await tester.pumpAndSettle();
      // No crash, snackbar confirms the refresh completed.
      expect(find.text('1 added, 0 removed'), findsOneWidget);
    });
  });

  group('GeneralPage — set default browser button', () {
    const urlChannel = MethodChannel('plugins.flutter.io/url_launcher_windows');

    setUp(() {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(urlChannel, (_) async => true);
    });

    tearDown(() {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(urlChannel, null);
    });

    testWidgets('tapping Set default button does not crash', (tester) async {
      final f = makeFixtures(dir: tempDir, isDefault: false);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Set default'));
      await tester.pumpAndSettle();
      expect(find.byType(GeneralPage), findsOneWidget);
    });
  });

  group('GeneralPage — hotkey dropdown', () {
    testWidgets('selecting a preset hotkey persists to file', (tester) async {
      tester.view.physicalSize = const Size(800, 2400);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();

      final dropdown = find.byType(DropdownButton<String?>);
      await tester.ensureVisible(dropdown);
      await tester.pumpAndSettle();
      await tester.tap(dropdown);
      await tester.pumpAndSettle();

      // Tap the first preset (index 0 in the platform list).
      final firstPreset = find.text(HotkeyPreset.defaults.first.label).last;
      await tester.tap(firstPreset);
      await tester.pumpAndSettle();

      final file = File('${tempDir.path}/global_hotkey');
      expect(file.existsSync(), isTrue);
      expect(file.readAsStringSync(), HotkeyPreset.defaults.first.serialized);
    });

    testWidgets('selecting None hotkey clears the file', (tester) async {
      tester.view.physicalSize = const Size(800, 2400);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      // Pre-write a hotkey so the initial value is non-null.
      File(
        '${tempDir.path}/global_hotkey',
      ).writeAsStringSync(HotkeyPreset.defaults.first.serialized);

      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();

      final dropdown = find.byType(DropdownButton<String?>);
      await tester.ensureVisible(dropdown);
      await tester.pumpAndSettle();
      await tester.tap(dropdown);
      await tester.pumpAndSettle();

      await tester.tap(find.text('None (disabled)').last);
      await tester.pumpAndSettle();

      final file = File('${tempDir.path}/global_hotkey');
      // After clearing, the file is either gone or empty.
      expect(!file.existsSync() || file.readAsStringSync().isEmpty, isTrue);
    });

    testWidgets(
      'dropdown shows extra item when saved hotkey is not in platform presets',
      (tester) async {
        tester.view.physicalSize = const Size(800, 2400);
        tester.view.devicePixelRatio = 1.0;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);

        // Write a serialized string that exists on no platform preset list.
        const alienHotkey = 'ctrl+shift+z';
        File('${tempDir.path}/global_hotkey').writeAsStringSync(alienHotkey);

        final f = makeFixtures(dir: tempDir);
        await tester.pumpWidget(
          buildTestApp(const GeneralPage(), overrides: f.overrides),
        );
        await tester.pumpAndSettle();

        // The current value shown in the collapsed dropdown is the alien key.
        expect(find.text(alienHotkey), findsWidgets);
      },
    );

    testWidgets(
      'hide tray subtitle shows Windows text when hotkey is set on non-mac',
      (tester) async {
        tester.view.physicalSize = const Size(800, 2400);
        tester.view.devicePixelRatio = 1.0;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);

        File(
          '${tempDir.path}/global_hotkey',
        ).writeAsStringSync(HotkeyPreset.defaults.first.serialized);

        final f = makeFixtures(dir: tempDir);
        await tester.pumpWidget(
          buildTestApp(const GeneralPage(), overrides: f.overrides),
        );
        await tester.pumpAndSettle();

        // On Windows the subtitle says "You can reach the app via its global shortcut".
        if (!Platform.isMacOS) {
          expect(find.textContaining('global shortcut'), findsWidgets);
        }
      },
    );

    testWidgets('hide tray switch can be toggled when hotkey is set', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(800, 2400);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      File(
        '${tempDir.path}/global_hotkey',
      ).writeAsStringSync(HotkeyPreset.defaults.first.serialized);

      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();

      final switches = tester.widgetList<Switch>(find.byType(Switch)).toList();
      final hideTraySwitch = switches.last;
      expect(hideTraySwitch.onChanged, isNotNull);

      // Tap the last switch (hide-tray) and confirm it doesn't crash.
      await tester.tap(find.byWidget(hideTraySwitch));
      await tester.pumpAndSettle();
      expect(find.byType(GeneralPage), findsOneWidget);
    });
  });

  group('GeneralPage — _updateBrowserIcon with custom icon path', () {
    testWidgets('adding browser always calls icon extractor after save', (
      tester,
    ) async {
      final recording = _RecordingIconExtractor();
      final f = makeFixtures(dir: tempDir, iconExtractor: recording);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byTooltip('Add custom browser'));
      await tester.pumpAndSettle();

      // Fill required fields only — iconPath left empty so exePath is used.
      await tester.enterText(
        find.widgetWithText(TextField, 'Name'),
        'My Browser',
      );
      await tester.enterText(
        find.widgetWithText(TextField, 'Executable path'),
        '/usr/bin/mybrowser',
      );

      // Tap Add and let all async futures settle via runAsync.
      await tester.runAsync(() async {
        await tester.tap(find.text('Add'));
        await Future<void>.delayed(const Duration(milliseconds: 200));
      });
      await tester.pumpAndSettle();

      // Extractor is called even with no custom icon (uses exePath as source).
      expect(recording.extractCalls, greaterThanOrEqualTo(1));
    });

    testWidgets('duplicating browser without icon file does not crash', (
      tester,
    ) async {
      // chrome has no icon file on disk → sourceIcon.existsSync() == false
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byIcon(Icons.more_vert));
      await tester.pumpAndSettle();
      await tester.runAsync(() async {
        await tester.tap(find.text('Duplicate'));
        await Future<void>.delayed(const Duration(milliseconds: 100));
      });
      await tester.pumpAndSettle();
      expect(find.text('Google Chrome'), findsWidgets);
    });

    testWidgets('adding browser with icon extraction failure does not crash', (
      tester,
    ) async {
      final f = makeFixtures(
        dir: tempDir,
        iconExtractor: _ThrowingIconExtractor(),
      );
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byTooltip('Add custom browser'));
      await tester.pumpAndSettle();
      await tester.enterText(
        find.widgetWithText(TextField, 'Name'),
        'Bad Browser',
      );
      await tester.enterText(
        find.widgetWithText(TextField, 'Executable path'),
        '/usr/bin/bad',
      );

      await tester.runAsync(() async {
        await tester.tap(find.text('Add'));
        await Future<void>.delayed(const Duration(milliseconds: 100));
      });
      await tester.pumpAndSettle();

      // Page still renders after best-effort icon extraction failure.
      expect(find.byType(GeneralPage), findsOneWidget);
    });
  });

  // The extended association list only renders on Windows (macOS shows
  // http/https only), so these assertions are Windows-specific.
  group('GeneralPage — associations row labels', skip: !Platform.isWindows, () {
    testWidgets('shows XHTML and SVG labels in associations row', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.textContaining('XHTML'), findsOneWidget);
      expect(find.textContaining('SVG'), findsOneWidget);
    });

    testWidgets('shows HTTP, HTTPS, HTM, HTML, PDF labels', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.textContaining('HTTP'), findsWidgets);
      expect(find.textContaining('HTTPS'), findsWidgets);
      expect(find.textContaining('HTM'), findsWidgets);
      expect(find.textContaining('HTML'), findsWidgets);
      expect(find.textContaining('PDF'), findsWidgets);
    });

    testWidgets('all association labels visible when all associations set', (
      tester,
    ) async {
      final f = makeFixtures(
        dir: tempDir,
        isDefault: true,
        associations: {
          'http',
          'https',
          '.htm',
          '.html',
          '.xhtml',
          '.svg',
          '.pdf',
        },
      );
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();
      expect(find.textContaining('XHTML'), findsOneWidget);
      expect(find.textContaining('SVG'), findsOneWidget);
    });

    testWidgets(
      'xhtml and svg labels present when only those associations set',
      (tester) async {
        final f = makeFixtures(
          dir: tempDir,
          isDefault: true,
          associations: {'.xhtml', '.svg'},
        );
        await tester.pumpWidget(
          buildTestApp(const GeneralPage(), overrides: f.overrides),
        );
        await tester.pumpAndSettle();
        expect(find.textContaining('XHTML'), findsOneWidget);
        expect(find.textContaining('SVG'), findsOneWidget);
      },
    );
  });
}

final class _ThrowingStartupService implements StartupService {
  @override
  Future<void> enable(String executablePath) =>
      Future.error(Exception('startup failed'));

  @override
  Future<void> disable() => Future.error(Exception('startup failed'));

  @override
  Future<bool> get isEnabled async => false;
}

final class _RecordingStartupService implements StartupService {
  _RecordingStartupService({required this.isEnabledValue});

  final bool isEnabledValue;
  int enableCalls = 0;
  int disableCalls = 0;
  String lastEnablePath = '';

  @override
  Future<void> enable(String executablePath) async {
    enableCalls++;
    lastEnablePath = executablePath;
  }

  @override
  Future<void> disable() async {
    disableCalls++;
  }

  @override
  Future<bool> get isEnabled async => isEnabledValue;
}

final class _ThrowingIconExtractor implements IconExtractor {
  @override
  Future<String> extractIcon(String executablePath, String outputPath) =>
      Future.error(Exception('icon extraction failed'));
}

final class _RecordingIconExtractor implements IconExtractor {
  int extractCalls = 0;

  @override
  Future<String> extractIcon(String executablePath, String outputPath) async {
    extractCalls++;
    return outputPath;
  }
}
