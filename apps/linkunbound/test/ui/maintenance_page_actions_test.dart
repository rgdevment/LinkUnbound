import 'dart:io';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:package_info_plus/package_info_plus.dart';

import 'package:linkunbound/providers.dart';
import 'package:linkunbound/ui/settings/maintenance_page.dart';

import '../helpers.dart';

const _chrome = Browser(
  id: 'chrome',
  name: 'Google Chrome',
  executablePath:
      '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
  iconPath: 'chrome.png',
);

final class _RecordingRegistrationService implements RegistrationService {
  int unregisterCalls = 0;

  @override
  Future<Set<String>> get defaultAssociations async => {};

  @override
  Future<bool> get isDefault async => false;

  @override
  Future<void> register(String executablePath) async {}

  @override
  Future<void> unregister() async {
    unregisterCalls++;
  }
}

final class _RecordingIconExtractor implements IconExtractor {
  final List<(String executablePath, String outputPath)> calls = [];

  @override
  Future<String> extractIcon(String executablePath, String outputPath) async {
    calls.add((executablePath, outputPath));
    return outputPath;
  }
}

List<Override> _makeOverrides({
  required Directory tempDir,
  required BrowserService browserService,
  required RuleService ruleService,
  required RegistrationService registrationService,
  required IconExtractor iconExtractor,
}) {
  final iconsDir = Directory('${tempDir.path}/icons')..createSync();

  return [
    browserServiceProvider.overrideWithValue(browserService),
    ruleServiceProvider.overrideWithValue(ruleService),
    registrationServiceProvider.overrideWithValue(registrationService),
    startupServiceProvider.overrideWithValue(FakeStartupService()),
    launchServiceProvider.overrideWithValue(FakeLaunchService()),
    iconExtractorProvider.overrideWithValue(iconExtractor),
    iconsDirProvider.overrideWithValue(iconsDir),
    localeFileProvider.overrideWithValue(File('${tempDir.path}/locale')),
    edgeWarningFileProvider.overrideWithValue(
      File('${tempDir.path}/edge_warning_dismissed'),
    ),
    appDataDirProvider.overrideWithValue(tempDir),
    packageInfoProvider.overrideWith(
      (ref) async => PackageInfo(
        appName: 'LinkUnbound',
        packageName: 'linkunbound',
        version: '1.0.0',
        buildNumber: '1',
        buildSignature: 'sig',
      ),
    ),
    updateInfoProvider.overrideWith((ref) async => null),
  ];
}

void main() {
  late Directory tempDir;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync(
      'maintenance_page_actions_test_',
    );
  });

  tearDown(() {
    if (tempDir.existsSync()) {
      tempDir.deleteSync(recursive: true);
    }
  });

  testWidgets('confirming reset rescans browsers and re-extracts icons', (
    tester,
  ) async {
    final browserService =
        BrowserService(
          configFile: File('${tempDir.path}/browsers.json'),
          browserDetector: FakeBrowserDetector([_chrome]),
        )..addBrowser(
          const Browser(
            id: 'old',
            name: 'Old Browser',
            executablePath: '/tmp/old-browser',
            iconPath: 'old.png',
          ),
        );
    final ruleService = RuleService(
      rulesFile: File('${tempDir.path}/rules.json'),
    );
    final registration = _RecordingRegistrationService();
    final iconExtractor = _RecordingIconExtractor();

    await tester.pumpWidget(
      buildTestApp(
        const MaintenancePage(),
        overrides: _makeOverrides(
          tempDir: tempDir,
          browserService: browserService,
          ruleService: ruleService,
          registrationService: registration,
          iconExtractor: iconExtractor,
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Reset configuration'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Reset'));
    await tester.pumpAndSettle();

    expect(browserService.browsers.map((browser) => browser.id).toList(), [
      'chrome',
    ]);
    expect(iconExtractor.calls, hasLength(1));
    expect(iconExtractor.calls.single.$1, _chrome.executablePath);
    expect(iconExtractor.calls.single.$2, endsWith('${Platform.pathSeparator}chrome.png'));
  });

  testWidgets('confirming unregister calls registration service', (
    tester,
  ) async {
    final browserService = BrowserService(
      configFile: File('${tempDir.path}/browsers.json'),
      browserDetector: FakeBrowserDetector(),
    );
    final ruleService = RuleService(
      rulesFile: File('${tempDir.path}/rules.json'),
    );
    final registration = _RecordingRegistrationService();

    await tester.pumpWidget(
      buildTestApp(
        const MaintenancePage(),
        overrides: _makeOverrides(
          tempDir: tempDir,
          browserService: browserService,
          ruleService: ruleService,
          registrationService: registration,
          iconExtractor: FakeIconExtractor(),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Unregister LinkUnbound'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Unregister'));
    await tester.pumpAndSettle();

    expect(registration.unregisterCalls, 1);
  });
}
