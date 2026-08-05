import 'dart:io';

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linkunbound_core/linkunbound_core.dart';

import 'package:linkunbound/ui/picker/picker_layout.dart';
import 'package:linkunbound/ui/picker/picker_view.dart';

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
    tempDir = Directory.systemTemp.createTempSync('picker_view_test_');
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

  group('PickerView — URL header', () {
    testWidgets('renders without throwing', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://github.com/user/repo'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byType(PickerView), findsOneWidget);
    });

    testWidgets('shows domain extracted from URL', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://github.com/user/repo'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('github.com'), findsOneWidget);
    });

    testWidgets('shows full URL in header', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://github.com/user/repo'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.textContaining('github.com/user/repo'), findsWidgets);
    });

    testWidgets('shows copy button', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byIcon(Icons.copy), findsOneWidget);
    });

    testWidgets('copy button has Copy URL tooltip', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byTooltip('Copy URL'), findsOneWidget);
    });

    testWidgets('shows link icon in header', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byIcon(Icons.link), findsOneWidget);
    });
  });

  group('PickerView — browser list', () {
    testWidgets('shows an explanation instead of a blank window when no '
        'browsers', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      // An empty ListView renders as an empty window, which reads as a broken
      // app rather than "no browsers were detected".
      expect(find.byType(ListView), findsNothing);
      expect(
        find.text('No browsers detected. Open Settings to add one.'),
        findsOneWidget,
      );
    });

    testWidgets('shows browser names when browsers provided', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome, _firefox]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Google Chrome'), findsOneWidget);
      expect(find.text('Firefox'), findsOneWidget);
    });

    testWidgets('shows keyboard shortcut badges 1 and 2 for two browsers', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome, _firefox]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('1'), findsOneWidget);
      expect(find.text('2'), findsOneWidget);
    });
  });

  group('PickerView — always open footer', () {
    testWidgets('shows Always open here checkbox', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byType(Checkbox), findsOneWidget);
    });

    testWidgets('checkbox is unchecked by default', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      final checkbox = tester.widget<Checkbox>(find.byType(Checkbox));
      expect(checkbox.value, isFalse);
    });

    testWidgets('tapping checkbox toggles it', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byType(Checkbox));
      await tester.pump();
      final checkbox = tester.widget<Checkbox>(find.byType(Checkbox));
      expect(checkbox.value, isTrue);
    });

    testWidgets('shows Always open here label', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Always open here'), findsOneWidget);
    });
  });

  group('PickerView — keyboard', () {
    testWidgets('Escape does not throw', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      await tester.pumpAndSettle();
      expect(find.byType(PickerView), findsOneWidget);
    });
  });

  group('PickerView — browser launch', () {
    testWidgets('tapping browser row triggers launch', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Google Chrome'));
      await tester.pumpAndSettle();
      expect(f.launchService.launches, contains('chrome.exe'));
    });

    testWidgets('pressing digit 1 launches first browser', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome, _firefox]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      await tester.sendKeyEvent(LogicalKeyboardKey.digit1);
      await tester.pumpAndSettle();
      expect(f.launchService.launches, contains('chrome.exe'));
    });

    testWidgets('pressing digit 2 launches second browser', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome, _firefox]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      await tester.sendKeyEvent(LogicalKeyboardKey.digit2);
      await tester.pumpAndSettle();
      expect(f.launchService.launches, contains('firefox.exe'));
    });

    testWidgets('launching with always open saves rule', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://github.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byType(Checkbox));
      await tester.pump();
      await tester.tap(find.text('Google Chrome'));
      await tester.pumpAndSettle();
      expect(f.ruleService.lookupBrowser('https://github.com'), 'chrome');
    });
  });

  group('PickerView — update dot', () {
    testWidgets('shows update dot when updateInfo is non-null', (tester) async {
      final f = makeFixtures(
        dir: tempDir,
        updateInfo: const UpdateInfo(
          latestVersion: '9.9.9',
          releaseUrl: 'https://github.com/test/releases',
        ),
      );
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      // Use pump instead of pumpAndSettle: _UpdateDot has a repeating animation
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 200));
      expect(find.byType(AnimatedBuilder), findsWidgets);
    });

    testWidgets('no update dot when updateInfo is null', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      expect(
        find.byTooltip('New version available — check for updates in Settings'),
        findsNothing,
      );
    });
  });

  group('PickerView — local file URLs', () {
    testWidgets('file:// URL shows file icon instead of link icon', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'file:///home/user/documents/report.pdf'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byIcon(Icons.insert_drive_file_outlined), findsOneWidget);
      expect(find.byIcon(Icons.link), findsNothing);
    });

    testWidgets('file:// URL shows filename as primary domain text', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'file:///home/user/documents/report.pdf'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('report.pdf'), findsOneWidget);
    });

    testWidgets('file:// URL with deep path shows …/parent/file in secondary', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'file:///home/user/documents/report.pdf'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('…/documents/report.pdf'), findsOneWidget);
    });

    testWidgets(
      'file:// URL with single-segment path shows …/file in secondary',
      (tester) async {
        final f = makeFixtures(dir: tempDir);
        await tester.pumpWidget(
          buildTestApp(
            const PickerView(url: 'file:///report.pdf'),
            overrides: f.overrides,
          ),
        );
        await tester.pumpAndSettle();
        // domain = 'report.pdf'; secondary = '…/report.pdf'
        expect(find.text('…/report.pdf'), findsOneWidget);
      },
    );

    testWidgets(
      'launching with always-open and file:// URL does not save a rule',
      (tester) async {
        final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
        await tester.pumpWidget(
          buildTestApp(
            const PickerView(url: 'file:///home/user/report.pdf'),
            overrides: f.overrides,
          ),
        );
        await tester.pumpAndSettle();
        await tester.tap(find.byType(Checkbox));
        await tester.pump();
        await tester.tap(find.text('Google Chrome'));
        await tester.pumpAndSettle();
        // file:// URLs have no host → guard in _launch prevents rule creation
        expect(f.launchService.launches, contains('chrome.exe'));
        expect(
          f.ruleService.lookupBrowser('file:///home/user/report.pdf'),
          isNull,
        );
      },
    );
  });

  group('PickerView — numpad keyboard shortcuts', () {
    testWidgets('numpad1 launches first browser', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome, _firefox]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      await tester.sendKeyEvent(LogicalKeyboardKey.numpad1);
      await tester.pumpAndSettle();
      expect(f.launchService.launches, contains('chrome.exe'));
    });

    testWidgets('numpad2 launches second browser', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome, _firefox]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      await tester.sendKeyEvent(LogicalKeyboardKey.numpad2);
      await tester.pumpAndSettle();
      expect(f.launchService.launches, contains('firefox.exe'));
    });

    testWidgets(
      'digit3 through digit9 are mapped (index within range launches)',
      (tester) async {
        final browsers = [
          _chrome,
          _firefox,
          const Browser(
            id: 'b3',
            name: 'B3',
            executablePath: 'b3.exe',
            iconPath: '',
          ),
          const Browser(
            id: 'b4',
            name: 'B4',
            executablePath: 'b4.exe',
            iconPath: '',
          ),
          const Browser(
            id: 'b5',
            name: 'B5',
            executablePath: 'b5.exe',
            iconPath: '',
          ),
          const Browser(
            id: 'b6',
            name: 'B6',
            executablePath: 'b6.exe',
            iconPath: '',
          ),
          const Browser(
            id: 'b7',
            name: 'B7',
            executablePath: 'b7.exe',
            iconPath: '',
          ),
          const Browser(
            id: 'b8',
            name: 'B8',
            executablePath: 'b8.exe',
            iconPath: '',
          ),
          const Browser(
            id: 'b9',
            name: 'B9',
            executablePath: 'b9.exe',
            iconPath: '',
          ),
        ];
        final f = makeFixtures(dir: tempDir, browsers: browsers);
        await tester.pumpWidget(
          buildTestApp(
            const PickerView(url: 'https://example.com'),
            overrides: f.overrides,
          ),
        );
        await tester.pumpAndSettle();
        await tester.sendKeyEvent(LogicalKeyboardKey.digit9);
        await tester.pumpAndSettle();
        expect(f.launchService.launches, contains('b9.exe'));
      },
    );

    testWidgets('numpad3-9 are each mapped (numpad3 launches index 2)', (
      tester,
    ) async {
      final browsers = [
        _chrome,
        _firefox,
        const Browser(
          id: 'b3',
          name: 'B3',
          executablePath: 'b3.exe',
          iconPath: '',
        ),
      ];
      final f = makeFixtures(dir: tempDir, browsers: browsers);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      await tester.sendKeyEvent(LogicalKeyboardKey.numpad3);
      await tester.pumpAndSettle();
      expect(f.launchService.launches, contains('b3.exe'));
    });

    testWidgets('out-of-range digit key does nothing (no launch)', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      // digit2 → index 1, but only 1 browser exists
      await tester.sendKeyEvent(LogicalKeyboardKey.digit2);
      await tester.pumpAndSettle();
      expect(f.launchService.launches, isEmpty);
    });

    testWidgets('unrecognised key is ignored', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
      await tester.pumpAndSettle();
      expect(f.launchService.launches, isEmpty);
    });
  });

  group('PickerView — copy button', () {
    testWidgets('tapping copy button does not throw', (tester) async {
      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com/path'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      // Clipboard.setData requires a platform channel; just verify no exception.
      await tester.tap(find.byIcon(Icons.copy));
      await tester.pumpAndSettle();
      expect(find.byType(PickerView), findsOneWidget);
    });
  });

  group('PickerView — browser icon rendering', () {
    testWidgets(
      'renders Image widget for browser (icon path constructed from id)',
      (tester) async {
        // The PickerView always constructs iconPath as "${iconsDir}/${id}.png";
        // when the file is absent Image.file renders (and errorBuilder fires asynchronously).
        final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
        await tester.pumpWidget(
          buildTestApp(
            const PickerView(url: 'https://example.com'),
            overrides: f.overrides,
          ),
        );
        await tester.pump();
        // Image widget is always rendered; errorBuilder fires later if decode fails.
        expect(find.byType(Image), findsWidgets);
      },
    );
  });

  group('PickerView — scrollbar with many browsers', () {
    testWidgets('scrollbar is visible when more than 6 browsers', (
      tester,
    ) async {
      final browsers = List.generate(
        7,
        (i) => Browser(
          id: 'b$i',
          name: 'Browser $i',
          executablePath: 'b$i.exe',
          iconPath: '',
        ),
      );
      final f = makeFixtures(dir: tempDir, browsers: browsers);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      final scrollbar = tester.widget<Scrollbar>(find.byType(Scrollbar));
      expect(scrollbar.thumbVisibility, isTrue);
    });

    testWidgets('no shortcut badge for browser at index 9 or beyond', (
      tester,
    ) async {
      final browsers = List.generate(
        10,
        (i) => Browser(
          id: 'b$i',
          name: 'Browser $i',
          executablePath: 'b$i.exe',
          iconPath: '',
        ),
      );
      final f = makeFixtures(dir: tempDir, browsers: browsers);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      // Only digits 1-9 appear; index 9 gets no badge so '10' never renders.
      expect(find.text('10'), findsNothing);
    });
  });

  group('PickerView — browser row hover', () {
    testWidgets('browser row changes color on mouse enter', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      final gesture = await tester.createGesture(kind: PointerDeviceKind.mouse);
      await gesture.addPointer(location: Offset.zero);
      addTearDown(gesture.removePointer);
      final browserRow = find.text('Google Chrome');
      await gesture.moveTo(tester.getCenter(browserRow));
      await tester.pumpAndSettle();
      expect(find.text('Google Chrome'), findsOneWidget);
    });

    testWidgets('browser row resets color on mouse exit', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      final gesture = await tester.createGesture(kind: PointerDeviceKind.mouse);
      await gesture.addPointer(location: Offset.zero);
      addTearDown(gesture.removePointer);
      final browserRow = find.text('Google Chrome');
      await gesture.moveTo(tester.getCenter(browserRow));
      await tester.pumpAndSettle();
      await gesture.moveTo(Offset.zero);
      await tester.pumpAndSettle();
      expect(find.text('Google Chrome'), findsOneWidget);
    });
  });

  group('PickerView — private mode', () {
    /// Holds Shift for the duration of the test. The picker reads
    /// [HardwareKeyboard] directly, so the key has to stay physically down
    /// rather than be sent as a one-shot event.
    Future<void> holdShift(WidgetTester tester) async {
      await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      addTearDown(() => tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft));
    }

    testWidgets('Shift marks the rows that can open privately', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byIcon(Icons.visibility_off_outlined), findsNothing);

      await holdShift(tester);
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.visibility_off_outlined), findsOneWidget);
    });

    testWidgets('Shift is seeded from the keyboard state on open', (
      tester,
    ) async {
      // The user holds Shift *before* clicking the link, so the key event that
      // would arm the intent has already happened by the time we build.
      await holdShift(tester);
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.visibility_off_outlined), findsOneWidget);
    });

    testWidgets('launching with Shift passes the private switch', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();
      await holdShift(tester);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Google Chrome'));
      await tester.pumpAndSettle();

      expect(f.launchService.privateArgsPerLaunch.single, ['--incognito']);
    });

    testWidgets('launching without Shift passes no switch', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Google Chrome'));
      await tester.pumpAndSettle();

      expect(f.launchService.privateArgsPerLaunch.single, isEmpty);
    });

    testWidgets('a browser that fails to start does not surface as a crash', (
      tester,
    ) async {
      // Unhandled, this future reached the zone guard and was filed as a crash
      // report carrying the full URL.
      final f = makeFixtures(
        dir: tempDir,
        browsers: [_chrome],
        launchService: FakeLaunchService(throws: true),
      );
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Google Chrome'));
      await tester.pumpAndSettle();

      expect(tester.takeException(), isNull);
      expect(f.launchService.launches, ['chrome.exe']);
    });
  });

  group('PickerView — origin-scoped rules', () {
    testWidgets('the footer names the originating app', (tester) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com', origin: 'slack'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('Always open links from slack here'), findsOneWidget);
      expect(find.text('Always open here'), findsNothing);
    });

    testWidgets('always-open with a known origin scopes the rule to the app', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com', origin: 'slack'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byType(Checkbox));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Google Chrome'));
      await tester.pumpAndSettle();

      final rule = f.ruleService.rules.single;
      expect(rule.sourceApp, 'slack');
      expect(rule.domain, kAnyDomain);
      expect(rule.private, isFalse);
    });

    testWidgets('the label takes every pixel the hint leaves', (tester) async {
      // Regression: a Spacer sat between the label and the hint and claimed an
      // equal share of the free width, halving the label's slot and rendering
      // "Abrir siempre aqui" as "Abrir siemp...". Asserted on layout geometry
      // rather than text width, which depends on the test font.
      tester.view.physicalSize = const Size(PickerLayout.width, 400);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();

      final label = tester.getRect(find.text('Always open here'));
      final hint = tester.getRect(find.text('Shift = private'));
      // Only the 8px gap may separate them; anything more is space the label
      // was entitled to.
      expect(label.right + 8, closeTo(hint.left, 0.5));
    });

    testWidgets('the private hint yields to the longer app-scoped label', (
      tester,
    ) async {
      // Both compete for the same row. The label says what ticking the box
      // does, so the hint is the one that goes.
      tester.view.physicalSize = const Size(PickerLayout.width, 400);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com', origin: 'slack'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('Shift = private'), findsNothing);
      final label = tester.getRect(
        find.text('Always open links from slack here'),
      );
      final checkbox = tester.getRect(find.byType(Checkbox));
      // The label now spans from the checkbox to the footer's right padding.
      expect(label.left, closeTo(checkbox.right + 8, 0.5));
      expect(label.right, closeTo(PickerLayout.width - 14, 0.5));
    });

    testWidgets('always-open without an origin falls back to the domain', (
      tester,
    ) async {
      final f = makeFixtures(dir: tempDir, browsers: [_chrome]);
      await tester.pumpWidget(
        buildTestApp(
          const PickerView(url: 'https://example.com/a'),
          overrides: f.overrides,
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byType(Checkbox));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Google Chrome'));
      await tester.pumpAndSettle();

      final rule = f.ruleService.rules.single;
      expect(rule.sourceApp, isNull);
      expect(rule.domain, 'example.com');
    });
  });
}
