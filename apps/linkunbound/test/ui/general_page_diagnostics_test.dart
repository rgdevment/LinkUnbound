import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linkunbound_core/linkunbound_core.dart';

import 'package:linkunbound/providers.dart';
import 'package:linkunbound/ui/settings/general_page.dart';

import '../helpers.dart';

/// The sections under test sit at the bottom of a ListView, so the viewport
/// has to be tall enough for them to be laid out at all.
void _useTallViewport(WidgetTester tester) {
  tester.view.physicalSize = const Size(800, 2400);
  tester.view.devicePixelRatio = 1.0;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);
}

const _staleHandler = HandlerDiagnostics(
  isDefaultBrowser: true,
  commandMatchesExecutable: false,
  runningFromDevBuild: false,
  isPackaged: false,
  recordedCommand: r'"C:\gone\linkunbound.exe" "%1"',
);

void main() {
  late Directory tempDir;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync('general_diag_test_');
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

  group('GeneralPage — diagnostics card', () {
    testWidgets('stays hidden while the registration is healthy', (
      tester,
    ) async {
      final f = makeFixtures(
        dir: tempDir,
        isDefault: true,
        registrationService: FakeRegistrationService(
          isDefaultValue: true,
          diagnostics: const HandlerDiagnostics(
            isDefaultBrowser: true,
            commandMatchesExecutable: true,
            runningFromDevBuild: false,
            isPackaged: false,
          ),
        ),
      );
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();

      expect(find.text('Link capture problem detected'), findsNothing);
    });

    testWidgets('stays hidden when the only problem is not being default', (
      tester,
    ) async {
      // The section right above already says so; repeating it as an error adds
      // noise, not information.
      final f = makeFixtures(
        dir: tempDir,
        registrationService: FakeRegistrationService(
          diagnostics: const HandlerDiagnostics(
            isDefaultBrowser: false,
            commandMatchesExecutable: true,
            runningFromDevBuild: false,
            isPackaged: false,
          ),
        ),
      );
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();

      expect(find.text('Link capture problem detected'), findsNothing);
    });

    testWidgets('explains a stale handler and offers a repair', (tester) async {
      _useTallViewport(tester);
      final f = makeFixtures(
        dir: tempDir,
        isDefault: true,
        registrationService: FakeRegistrationService(
          isDefaultValue: true,
          diagnostics: _staleHandler,
        ),
      );
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();

      expect(find.text('Link capture problem detected'), findsOneWidget);
      expect(
        find.textContaining('registered handler points somewhere else'),
        findsOneWidget,
      );
      expect(find.text('Repair'), findsOneWidget);
    });

    testWidgets('a build tree is explained but not offered a repair', (
      tester,
    ) async {
      // Registering a build tree would make things worse, not better.
      _useTallViewport(tester);
      final f = makeFixtures(
        dir: tempDir,
        isDefault: true,
        registrationService: FakeRegistrationService(
          isDefaultValue: true,
          diagnostics: const HandlerDiagnostics(
            isDefaultBrowser: true,
            commandMatchesExecutable: false,
            runningFromDevBuild: true,
            isPackaged: false,
          ),
        ),
      );
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();

      expect(find.textContaining('Running from a local build'), findsOneWidget);
      expect(find.text('Repair'), findsNothing);
    });

    testWidgets('Repair re-registers and confirms', (tester) async {
      _useTallViewport(tester);
      final registration = FakeRegistrationService(
        isDefaultValue: true,
        diagnostics: _staleHandler,
      );
      final f = makeFixtures(
        dir: tempDir,
        isDefault: true,
        registrationService: registration,
      );
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Repair'));
      await tester.pumpAndSettle();

      expect(registration.registrations, isNotEmpty);
      expect(find.text('Registration repaired.'), findsOneWidget);
    });

    testWidgets('a failed repair says so instead of claiming success', (
      tester,
    ) async {
      _useTallViewport(tester);
      final registration = FakeRegistrationService(
        isDefaultValue: true,
        diagnostics: _staleHandler,
        registerThrows: true,
      );
      final f = makeFixtures(
        dir: tempDir,
        isDefault: true,
        registrationService: registration,
      );
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Repair'));
      await tester.pumpAndSettle();

      expect(find.text('Could not repair the registration.'), findsOneWidget);
    });
  });

  group('GeneralPage — internal links section', () {
    const urlChannel = MethodChannel('plugins.flutter.io/url_launcher_windows');

    Future<void> pumpPage(
      WidgetTester tester, {
      required FakeRegistrationService registration,
      bool supported = true,
    }) async {
      _useTallViewport(tester);
      final f = makeFixtures(
        dir: tempDir,
        isDefault: true,
        registrationService: registration,
      );
      await tester.pumpWidget(
        buildTestApp(
          const GeneralPage(),
          overrides: [
            ...f.overrides,
            edgeProtocolSupportedProvider.overrideWithValue(supported),
          ],
        ),
      );
      await tester.pumpAndSettle();
    }

    testWidgets('is hidden where the scheme cannot be claimed', (tester) async {
      await pumpPage(
        tester,
        registration: FakeRegistrationService(isDefaultValue: true),
        supported: false,
      );
      expect(find.text('Capture links from Microsoft apps'), findsNothing);
    });

    testWidgets('reflects the current capture state', (tester) async {
      await pumpPage(
        tester,
        registration: FakeRegistrationService(
          isDefaultValue: true,
          capturesEdgeProtocolValue: true,
        ),
      );

      expect(find.text('Capture links from Microsoft apps'), findsOneWidget);
      final toggle = tester.widget<Switch>(
        find.descendant(
          of: find
              .ancestor(
                of: find.textContaining('Teams, Outlook and Start menu'),
                matching: find.byType(Row),
              )
              .first,
          matching: find.byType(Switch),
        ),
      );
      expect(toggle.value, isTrue);
    });

    testWidgets('enabling it registers the scheme', (tester) async {
      final registration = FakeRegistrationService(isDefaultValue: true);
      await pumpPage(tester, registration: registration);

      final toggle = find.descendant(
        of: find
            .ancestor(
              of: find.textContaining('Teams, Outlook and Start menu'),
              matching: find.byType(Row),
            )
            .first,
        matching: find.byType(Switch),
      );
      await tester.tap(toggle);
      await tester.pumpAndSettle();

      expect(registration.edgeCaptureCalls, [true]);
    });

    testWidgets('a rejected registry write does not take the page down', (
      tester,
    ) async {
      final registration = FakeRegistrationService(
        isDefaultValue: true,
        edgeCaptureThrows: true,
      );
      await pumpPage(tester, registration: registration);

      final toggle = find.descendant(
        of: find
            .ancestor(
              of: find.textContaining('Teams, Outlook and Start menu'),
              matching: find.byType(Row),
            )
            .first,
        matching: find.byType(Switch),
      );
      await tester.tap(toggle);
      await tester.pumpAndSettle();

      expect(registration.edgeCaptureCalls, [true]);
      expect(tester.takeException(), isNull);
      expect(find.byType(GeneralPage), findsOneWidget);
    });

    testWidgets('a failed re-registration still opens system settings', (
      tester,
    ) async {
      // "Set default" repairs the registration first, because opening the
      // system pane is useless when the recorded handler is stale. A refusal
      // there must not stop the pane from opening.
      final calls = <String>[];
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(urlChannel, (call) async {
            calls.add(call.method);
            return true;
          });
      addTearDown(
        () => TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(urlChannel, null),
      );

      final registration = FakeRegistrationService(registerThrows: true);
      final f = makeFixtures(dir: tempDir, registrationService: registration);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Set default'));
      await tester.pumpAndSettle();

      expect(registration.registrations, isNotEmpty);
      expect(find.byType(GeneralPage), findsOneWidget);
    });

    testWidgets('an unavailable settings pane is swallowed', (tester) async {
      // A locked-down machine can refuse to open ms-settings:. That must not
      // escape to the zone guard and be filed as a crash.
      for (final channel in const [
        MethodChannel('plugins.flutter.io/url_launcher'),
        MethodChannel('plugins.flutter.io/url_launcher_windows'),
      ]) {
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(channel, (_) async {
              throw PlatformException(code: 'ERROR', message: 'blocked');
            });
        addTearDown(
          () => TestDefaultBinaryMessengerBinding
              .instance
              .defaultBinaryMessenger
              .setMockMethodCallHandler(channel, null),
        );
      }

      final f = makeFixtures(dir: tempDir);
      await tester.pumpWidget(
        buildTestApp(const GeneralPage(), overrides: f.overrides),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Set default'));
      await tester.pumpAndSettle();

      expect(tester.takeException(), isNull);
      expect(find.byType(GeneralPage), findsOneWidget);
    });
  });
}
