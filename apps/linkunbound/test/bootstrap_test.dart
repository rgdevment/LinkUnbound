import 'dart:async';
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:package_info_plus/package_info_plus.dart';

import 'package:linkunbound/bootstrap.dart';
import 'package:linkunbound/platform/cursor_locator.dart';
import 'package:linkunbound/platform/platform_bindings.dart';
import 'package:linkunbound/platform/tray_controller.dart';
import 'package:linkunbound/ui/picker/picker_window.dart';
import 'package:linkunbound/ui/settings/settings_window.dart';

const _hotkeyChannel = MethodChannel('dev.leanflutter.plugins/hotkey_manager');

const _hotkeyEventChannel =
    MethodChannel('dev.leanflutter.plugins/hotkey_manager_event');

const _windowChannel = MethodChannel('window_manager');
const _macWindowChannel = MethodChannel('linkunbound/window');
const _screenChannel = MethodChannel(
  'dev.leanflutter.plugins/screen_retriever',
);

const _chrome = Browser(
  id: 'chrome',
  name: 'Google Chrome',
  executablePath:
      '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
  iconPath: 'chrome.png',
);

final class _MethodChannelSpy {
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
      case 'getBounds':
        // window_manager casts the result to Map<dynamic,dynamic> before
        // parsing; returning null causes a TypeError, so return a fake rect.
        return <String, dynamic>{
          'x': 0.0,
          'y': 0.0,
          'width': 800.0,
          'height': 600.0,
        };
      default:
        return null;
    }
  }
}

/// Mocks the screen_retriever channel that window_manager's center() uses
/// to locate the primary display and cursor position.
final class _ScreenSpy {
  final List<MethodCall> calls = [];

  static const _fakeDisplay = <String, dynamic>{
    'id': '1',
    'name': 'Test Display',
    'size': <String, dynamic>{'width': 1280.0, 'height': 800.0},
    'visiblePosition': <String, dynamic>{'dx': 0.0, 'dy': 0.0},
    'visibleSize': <String, dynamic>{'width': 1280.0, 'height': 800.0},
    'scaleFactor': 2.0,
  };

  Future<dynamic> handle(MethodCall call) async {
    calls.add(call);
    switch (call.method) {
      case 'getPrimaryDisplay':
        return _fakeDisplay;
      case 'getAllDisplays':
        return <String, dynamic>{
          'displays': [_fakeDisplay],
        };
      case 'getCursorScreenPoint':
        return <String, dynamic>{'dx': 640.0, 'dy': 400.0};
      default:
        return null;
    }
  }
}

final class _FailingHttpClient extends Fake implements HttpClient {
  @override
  Future<HttpClientRequest> getUrl(Uri url) {
    throw const SocketException('blocked in tests');
  }
}

final class _FakeBrowserDetector implements BrowserDetector {
  _FakeBrowserDetector(this.detectedBrowsers);

  final List<Browser> detectedBrowsers;

  @override
  Future<List<Browser>> detect() async => detectedBrowsers;
}

final class _RecordingIconExtractor implements IconExtractor {
  final List<(String executablePath, String outputPath)> calls = [];

  @override
  Future<String> extractIcon(String executablePath, String outputPath) async {
    calls.add((executablePath, outputPath));
    return outputPath;
  }
}

final class _RecordingRegistrationService implements RegistrationService {
  final List<String> registerCalls = [];

  @override
  Future<Set<String>> get defaultAssociations async => {};

  @override
  Future<bool> get isDefault async => false;

  @override
  Future<void> register(String executablePath) async {
    registerCalls.add(executablePath);
  }

  @override
  Future<void> unregister() async {}
}

final class _FakeStartupService implements StartupService {
  @override
  Future<void> disable() async {}

  @override
  Future<void> enable(String executablePath) async {}

  @override
  Future<bool> get isEnabled async => false;
}

final class _RecordingLaunchService implements LaunchService {
  final List<({String executablePath, String url, List<String> extraArgs})>
  calls = [];

  @override
  Future<void> launch(
    String executablePath,
    String url,
    List<String> extraArgs,
  ) async {
    calls.add((
      executablePath: executablePath,
      url: url,
      extraArgs: List<String>.from(extraArgs),
    ));
  }
}

final class _FakeCursorLocator implements CursorLocator {
  _FakeCursorLocator();

  final (double, double) cursor = const (300.0, 200.0);
  final (double, double) screen = const (1280.0, 900.0);

  @override
  Future<(double, double)> cursorPosition() async => cursor;

  @override
  Future<(double, double)> screenSize() async => screen;

  @override
  Future<List<({double originX, double originY, double width, double height})>>
  displayRects() async => [
    (originX: 0.0, originY: 0.0, width: screen.$1, height: screen.$2),
  ];
}

final class _FakeTrayController implements TrayController {
  int initCalls = 0;
  List<TrayMenuItem> menuItems = const [];
  VoidCallback? activationCallback;

  @override
  Future<void> dispose() async {}

  @override
  Future<void> init({
    required String title,
    required String iconPath,
    required String tooltip,
  }) async {
    initCalls++;
  }

  @override
  void onActivated(VoidCallback callback) {
    activationCallback = callback;
  }

  @override
  Future<void> setMenu(List<TrayMenuItem> items) async {
    menuItems = items;
  }

  void activate() {
    activationCallback?.call();
  }
}

final class _FakeBindings implements PlatformBindings {
  _FakeBindings({
    required this.rootDir,
    List<Browser> detectedBrowsers = const [],
    LaunchService? launchOverride,
  }) : browserDetector = _FakeBrowserDetector(detectedBrowsers),
       iconExtractor = _RecordingIconExtractor(),
       registrationService = _RecordingRegistrationService(),
       startupService = _FakeStartupService(),
       launchService = launchOverride ?? _RecordingLaunchService(),
       trayController = _FakeTrayController(),
       cursorLocator = _FakeCursorLocator(),
       _events = StreamController<InboundEvent>.broadcast() {
    appDataDir.createSync(recursive: true);
    iconsDir.createSync(recursive: true);
    trayIconPathFile.writeAsStringSync('icon');
  }

  final Directory rootDir;
  final InboundEvent? initial = null;
  final StreamController<InboundEvent> _events;

  @override
  final BrowserDetector browserDetector;

  @override
  final _RecordingIconExtractor iconExtractor;

  @override
  final LaunchService launchService;

  _RecordingLaunchService get launchRecorder =>
      launchService as _RecordingLaunchService;

  @override
  final _RecordingRegistrationService registrationService;

  @override
  final _FakeStartupService startupService;

  @override
  final _FakeTrayController trayController;

  @override
  final CursorLocator cursorLocator;

  int claimCalls = 0;
  int releaseCalls = 0;
  int tryDelegateCalls = 0;

  @override
  bool startsHidden = false;

  @override
  Directory get appDataDir => Directory('${rootDir.path}/app-data');

  @override
  File get browsersFile => File('${appDataDir.path}/browsers.json');

  @override
  File get edgeWarningFile => File('${appDataDir.path}/edge_warning_dismissed');

  @override
  String get executablePath =>
      '/Applications/LinkUnbound.app/Contents/MacOS/LinkUnbound';

  @override
  Directory get iconsDir => Directory('${appDataDir.path}/icons');

  @override
  InboundEvent? get initialEvent => initial;

  @override
  Stream<InboundEvent> get inboundEvents => _events.stream;

  @override
  File get localeFile => File('${appDataDir.path}/locale');

  @override
  File get logFile => File('${appDataDir.path}/linkunbound.log');

  @override
  File get rulesFile => File('${appDataDir.path}/rules.json');

  @override
  File get hideTrayFile => File('${appDataDir.path}/hide_tray');

  @override
  File get globalHotkeyFile => File('${appDataDir.path}/global_hotkey');

  File get trayIconPathFile => File('${appDataDir.path}/tray.png');

  @override
  String get trayIconPath => trayIconPathFile.path;

  @override
  Future<bool> claim() async {
    claimCalls++;
    return true;
  }

  Future<void> close() async {
    await _events.close();
  }

  Future<void> emit(InboundEvent event) async {
    // Broadcast streams deliver synchronously; callers pump the tester
    // themselves to drain any Riverpod state-change microtasks.
    _events.add(event);
  }

  Future<void> emitError(Object error) async {
    _events.addError(error);
  }

  @override
  Future<void> release() async {
    releaseCalls++;
  }

  Future<void> seed({
    List<Browser> browsers = const [],
    List<Rule> rules = const [],
  }) async {
    final browserService = BrowserService(
      configFile: browsersFile,
      browserDetector: browserDetector,
    );
    for (final browser in browsers) {
      browserService.addBrowser(browser);
    }
    await browserService.save();

    final ruleService = RuleService(rulesFile: rulesFile);
    for (final rule in rules) {
      ruleService.addRule(rule);
    }
    await ruleService.save();
  }

  @override
  Future<bool> tryDelegate(InboundEvent? event) async {
    tryDelegateCalls++;
    return false;
  }
}

final class _FailingLaunchService implements LaunchService {
  @override
  Future<void> launch(
    String executablePath,
    String url,
    List<String> extraArgs,
  ) => Future.error(Exception('launch failed'));
}

final class _ThrowingDelegateBindings extends _FakeBindings {
  _ThrowingDelegateBindings({required super.rootDir});

  @override
  Future<bool> tryDelegate(InboundEvent? event) async {
    tryDelegateCalls++;
    throw const SocketException('simulated delegation failure');
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  late Directory tempDir;
  late _MethodChannelSpy windowSpy;
  late _MethodChannelSpy macWindowSpy;
  late _ScreenSpy screenSpy;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync('bootstrap_test_');
    PackageInfo.setMockInitialValues(
      appName: 'LinkUnbound',
      packageName: 'dev.rg.LinkUnbound',
      version: '1.0.0',
      buildNumber: '1',
      buildSignature: 'sig',
    );
    windowSpy = _MethodChannelSpy();
    macWindowSpy = _MethodChannelSpy();
    screenSpy = _ScreenSpy();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_windowChannel, windowSpy.handle);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_macWindowChannel, macWindowSpy.handle);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_screenChannel, screenSpy.handle);
    // hotkey_manager calls into native on register/unregister; return null (no-op).
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_hotkeyChannel, (_) async => null);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_hotkeyEventChannel, (_) async => null);
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_windowChannel, null);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_macWindowChannel, null);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_screenChannel, null);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_hotkeyChannel, null);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(_hotkeyEventChannel, null);
    disposeLogging();
    if (tempDir.existsSync()) {
      tempDir.deleteSync(recursive: true);
    }
  });

  Future<void> boot(
    WidgetTester tester,
    _FakeBindings bindings,
    List<String> args,
  ) async {
    // bootstrap() performs real dart:io and platform-channel operations
    // (file reads, tray init, AppLocalizations.delegate.load, runApp) that
    // need the real event loop.  tester.runAsync escapes FakeAsync so those
    // futures can complete, then we pump to process widget frames.
    // The extra runAsync gives post-frame callbacks (tray init, icon
    // extraction) time to complete before we assert on their side effects.
    await tester.runAsync(() async {
      await HttpOverrides.runZoned(
        () => bootstrap(bindings, args),
        createHttpClient: (_) => _FailingHttpClient(),
      );
    });
    await tester.pump();
    await tester.pump();
    // Allow deferred post-frame async work (tray init, icon extraction) to
    // complete in real-time before widget assertions.
    await tester.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 100)),
    );
    await tester.pump();
    await tester.pump();
  }

  testWidgets('first boot scans browsers, extracts icons, and opens settings', (
    tester,
  ) async {
    final bindings = _FakeBindings(
      rootDir: tempDir,
      detectedBrowsers: const [_chrome],
    );
    addTearDown(bindings.close);

    await boot(tester, bindings, const []);

    expect(bindings.claimCalls, 1);
    expect(bindings.tryDelegateCalls, 1);
    expect(bindings.registrationService.registerCalls, [
      bindings.executablePath,
    ]);
    expect(bindings.iconExtractor.calls, hasLength(1));
    expect(bindings.iconExtractor.calls.single.$1, _chrome.executablePath);
    expect(bindings.trayController.initCalls, 1);
    expect(
      bindings.trayController.menuItems
          .map((item) => item.label)
          .whereType<String>(),
      containsAll(['Settings', 'Exit']),
    );
    expect(find.byType(SettingsWindow), findsOneWidget);
  });

  testWidgets('background launch stays hidden until tray activation', (
    tester,
  ) async {
    final bindings = _FakeBindings(rootDir: tempDir)..startsHidden = true;
    addTearDown(bindings.close);

    await boot(tester, bindings, const ['--background']);

    expect(find.byType(SettingsWindow), findsNothing);

    bindings.trayController.activate();
    await tester.pump();
    await tester.pump();

    expect(find.byType(SettingsWindow), findsOneWidget);
    if (Platform.isMacOS) {
      expect(macWindowSpy.methods, contains('setSettingsMode'));
      expect(macWindowSpy.methods, contains('activate'));
    }
  });

  testWidgets('matching rule launches browser instead of opening picker', (
    tester,
  ) async {
    final bindings = _FakeBindings(rootDir: tempDir)..startsHidden = true;
    addTearDown(bindings.close);
    await tester.runAsync(
      () => bindings.seed(
        browsers: const [_chrome],
        rules: const [Rule(domain: 'example.com', browserId: 'chrome')],
      ),
    );

    await boot(tester, bindings, const ['--background']);
    await bindings.emit(const OpenUrlEvent('https://example.com/docs'));
    await tester.pump();
    await tester.pump();

    expect(bindings.launchRecorder.calls, hasLength(1));
    expect(
      bindings.launchRecorder.calls.single.executablePath,
      _chrome.executablePath,
    );
    expect(
      bindings.launchRecorder.calls.single.url,
      'https://example.com/docs',
    );
    expect(find.byType(PickerWindow), findsNothing);
  });

  testWidgets('safe links are unwrapped before rule-based launch', (
    tester,
  ) async {
    final bindings = _FakeBindings(rootDir: tempDir)..startsHidden = true;
    addTearDown(bindings.close);
    await tester.runAsync(
      () => bindings.seed(
        browsers: const [_chrome],
        rules: const [Rule(domain: 'example.com', browserId: 'chrome')],
      ),
    );

    await boot(tester, bindings, const ['--background']);
    await bindings.emit(
      OpenUrlEvent(
        'https://nam12.safelinks.protection.outlook.com/?url=${Uri.encodeComponent('https://example.com/report?id=7')}',
      ),
    );
    await tester.pump();
    await tester.pump();

    expect(bindings.launchRecorder.calls, hasLength(1));
    expect(
      bindings.launchRecorder.calls.single.url,
      'https://example.com/report?id=7',
    );
  });

  testWidgets('valid local html file opens the picker', (tester) async {
    final bindings = _FakeBindings(
      rootDir: tempDir,
      detectedBrowsers: const [_chrome],
    )..startsHidden = true;
    addTearDown(bindings.close);
    final htmlFile = File('${tempDir.path}/preview.html')
      ..writeAsStringSync('<html></html>');

    await boot(tester, bindings, const ['--background']);
    // bootstrap() runs in tester.runAsync, so its listeners are in the real
    // event loop zone.  emit + a small real-time delay lets all the async
    // channel calls (setPickerMode, setSize, setPosition, show, …) complete
    // before tearDown removes the spy handlers.
    await tester.runAsync(() async {
      bindings.emit(OpenUrlEvent(htmlFile.uri.toString()));
      await Future<void>.delayed(const Duration(milliseconds: 100));
    });
    await tester.pump();
    await tester.pump();

    expect(find.byType(PickerWindow), findsOneWidget);
    if (Platform.isMacOS) {
      expect(macWindowSpy.methods, contains('setPickerMode'));
    }
  });

  testWidgets('unsupported local file is ignored', (tester) async {
    final bindings = _FakeBindings(rootDir: tempDir)..startsHidden = true;
    addTearDown(bindings.close);
    final txtFile = File('${tempDir.path}/notes.txt')..writeAsStringSync('hi');

    await boot(tester, bindings, const ['--background']);
    await bindings.emit(OpenUrlEvent(txtFile.uri.toString()));
    await tester.pump();
    await tester.pump();

    expect(bindings.launchRecorder.calls, isEmpty);
    expect(find.byType(PickerWindow), findsNothing);
  });

  testWidgets('ShowSettingsEvent opens settings window', (tester) async {
    final bindings = _FakeBindings(rootDir: tempDir)..startsHidden = true;
    addTearDown(bindings.close);

    await boot(tester, bindings, const ['--background']);
    expect(find.byType(SettingsWindow), findsNothing);

    await tester.runAsync(() async {
      bindings.emit(const ShowSettingsEvent());
      await Future<void>.delayed(const Duration(milliseconds: 100));
    });
    await tester.pump();
    await tester.pump();

    expect(find.byType(SettingsWindow), findsOneWidget);
    if (Platform.isMacOS) {
      expect(macWindowSpy.methods, contains('setSettingsMode'));
    }
  });

  testWidgets('tryDelegate exception is non-fatal and boot continues', (
    tester,
  ) async {
    final bindings = _ThrowingDelegateBindings(rootDir: tempDir);
    addTearDown(bindings.close);

    await boot(tester, bindings, const []);

    expect(bindings.tryDelegateCalls, 1);
    expect(find.byType(SettingsWindow), findsOneWidget);
  });

  testWidgets('corrupt browsers.json is reset and boot continues', (
    tester,
  ) async {
    final bindings = _FakeBindings(
      rootDir: tempDir,
      detectedBrowsers: const [_chrome],
    );
    addTearDown(bindings.close);

    bindings.browsersFile
      ..createSync(recursive: true)
      ..writeAsStringSync('{{not valid json');

    await boot(tester, bindings, const []);

    expect(find.byType(SettingsWindow), findsOneWidget);
  });

  testWidgets(
    'ShowSettingsEvent when settings already showing re-focuses window',
    (tester) async {
      final bindings = _FakeBindings(rootDir: tempDir)..startsHidden = false;
      addTearDown(bindings.close);

      await boot(tester, bindings, const []);
      expect(find.byType(SettingsWindow), findsOneWidget);

      windowSpy.clear();

      await tester.runAsync(() async {
        bindings.emit(const ShowSettingsEvent());
        await Future<void>.delayed(const Duration(milliseconds: 150));
      });
      await tester.pump();
      await tester.pump();

      expect(find.byType(SettingsWindow), findsOneWidget);
      expect(windowSpy.methods, contains('show'));
      expect(windowSpy.methods, contains('focus'));
    },
  );

  testWidgets('corrupt rules.json is ignored and boot continues', (
    tester,
  ) async {
    final bindings = _FakeBindings(rootDir: tempDir);
    addTearDown(bindings.close);
    bindings.rulesFile
      ..createSync(recursive: true)
      ..writeAsStringSync('{{not valid json');

    await boot(tester, bindings, const []);

    expect(find.byType(SettingsWindow), findsOneWidget);
  });

  testWidgets('no matching rule opens picker', (tester) async {
    final bindings = _FakeBindings(rootDir: tempDir)..startsHidden = true;
    addTearDown(bindings.close);
    await tester.runAsync(() => bindings.seed(browsers: const [_chrome]));

    await boot(tester, bindings, const ['--background']);
    await tester.runAsync(() async {
      bindings.emit(const OpenUrlEvent('https://no-rule-site.com/page'));
      await Future<void>.delayed(const Duration(milliseconds: 100));
    });
    await tester.pump();
    await tester.pump();

    expect(find.byType(PickerWindow), findsOneWidget);
    expect(bindings.launchRecorder.calls, isEmpty);
  });

  testWidgets('rule matched but browser missing opens picker', (tester) async {
    final bindings = _FakeBindings(rootDir: tempDir)..startsHidden = true;
    addTearDown(bindings.close);
    await tester.runAsync(
      () => bindings.seed(
        rules: const [Rule(domain: 'example.com', browserId: 'chrome')],
      ),
    );

    await boot(tester, bindings, const ['--background']);
    await tester.runAsync(() async {
      bindings.emit(const OpenUrlEvent('https://example.com/page'));
      await Future<void>.delayed(const Duration(milliseconds: 100));
    });
    await tester.pump();
    await tester.pump();

    expect(find.byType(PickerWindow), findsOneWidget);
    expect(bindings.launchRecorder.calls, isEmpty);
  });

  testWidgets('launch failure falls back to picker', (tester) async {
    final bindings = _FakeBindings(
      rootDir: tempDir,
      launchOverride: _FailingLaunchService(),
    )..startsHidden = true;
    addTearDown(bindings.close);
    await tester.runAsync(
      () => bindings.seed(
        browsers: const [_chrome],
        rules: const [Rule(domain: 'example.com', browserId: 'chrome')],
      ),
    );

    await boot(tester, bindings, const ['--background']);
    await tester.runAsync(() async {
      bindings.emit(const OpenUrlEvent('https://example.com/docs'));
      await Future<void>.delayed(const Duration(milliseconds: 200));
    });
    await tester.pump();
    await tester.pump();

    expect(find.byType(PickerWindow), findsOneWidget);
  });

  testWidgets('inbound events stream error is non-fatal', (tester) async {
    final bindings = _FakeBindings(rootDir: tempDir)..startsHidden = true;
    addTearDown(bindings.close);

    await boot(tester, bindings, const ['--background']);
    await tester.runAsync(() async {
      await bindings.emitError(Exception('stream error'));
      await Future<void>.delayed(const Duration(milliseconds: 50));
    });
    await tester.pump();

    expect(find.byType(SettingsWindow), findsNothing);
    expect(find.byType(PickerWindow), findsNothing);
  });
}
