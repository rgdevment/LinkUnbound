import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:package_info_plus/package_info_plus.dart';

import 'package:linkunbound/l10n/app_localizations.dart';
import 'package:linkunbound/providers.dart';

final class FakeRegistrationService implements RegistrationService {
  FakeRegistrationService({
    this.isDefaultValue = false,
    Set<String>? associations,
  }) : _associations = associations;

  final bool isDefaultValue;
  final Set<String>? _associations;

  @override
  Future<void> register(String executablePath) async {}

  @override
  Future<void> unregister() async {}

  @override
  Future<bool> get isDefault async => isDefaultValue;

  @override
  Future<Set<String>> get defaultAssociations async =>
      _associations ?? (isDefaultValue ? {'http', 'https'} : {});
}

final class FakeStartupService implements StartupService {
  FakeStartupService({this.isEnabledValue = false});
  final bool isEnabledValue;

  @override
  Future<void> enable(String executablePath) async {}

  @override
  Future<void> disable() async {}

  @override
  Future<bool> get isEnabled async => isEnabledValue;
}

final class FakeLaunchService implements LaunchService {
  final List<String> launches = [];

  @override
  Future<void> launch(
    String executablePath,
    String url,
    List<String> extraArgs,
  ) async {
    launches.add(executablePath);
  }
}

final class FakeIconExtractor implements IconExtractor {
  @override
  Future<String> extractIcon(String executablePath, String outputPath) async =>
      outputPath;
}

final class FakeBrowserDetector implements BrowserDetector {
  FakeBrowserDetector([this.browsers = const []]);
  final List<Browser> browsers;

  @override
  Future<List<Browser>> detect() async => browsers;
}

Widget buildTestApp(Widget child, {required List<Override> overrides}) {
  return ProviderScope(
    overrides: overrides,
    child: MaterialApp(
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: Scaffold(body: child),
    ),
  );
}

({
  List<Override> overrides,
  BrowserService browserService,
  RuleService ruleService,
  FakeLaunchService launchService,
  Directory tempDir,
})
makeFixtures({
  Directory? dir,
  List<Browser> browsers = const [],
  List<Browser> detectedBrowsers = const [],
  List<Rule> rules = const [],
  bool isDefault = false,
  Set<String>? associations,
  bool isStartupEnabled = false,
  StartupService? startupService,
  IconExtractor? iconExtractor,
  UpdateInfo? updateInfo,
  DiagnosticsExporter? diagnosticsExporter,
}) {
  final tempDir = dir ?? Directory.systemTemp.createTempSync('lu_test_');
  final configFile = File('${tempDir.path}/browsers.json');
  final rulesFile = File('${tempDir.path}/rules.json');
  final localeFile = File('${tempDir.path}/locale');
  final themeFile = File('${tempDir.path}/theme');
  final edgeWarningFile = File('${tempDir.path}/edge_warning_dismissed');
  final hideTrayFile = File('${tempDir.path}/hide_tray');
  final globalHotkeyFile = File('${tempDir.path}/global_hotkey');
  final iconsDir = Directory('${tempDir.path}/icons')..createSync();

  final browserService = BrowserService(
    configFile: configFile,
    browserDetector: FakeBrowserDetector(detectedBrowsers),
  );
  for (final b in browsers) {
    browserService.addBrowser(b);
  }

  final ruleService = RuleService(rulesFile: rulesFile);
  for (final r in rules) {
    ruleService.addRule(r);
  }

  final launchService = FakeLaunchService();

  final overrides = <Override>[
    browserServiceProvider.overrideWithValue(browserService),
    ruleServiceProvider.overrideWithValue(ruleService),
    registrationServiceProvider.overrideWithValue(
      FakeRegistrationService(
        isDefaultValue: isDefault,
        associations: associations,
      ),
    ),
    startupServiceProvider.overrideWithValue(
      startupService ?? FakeStartupService(isEnabledValue: isStartupEnabled),
    ),
    launchServiceProvider.overrideWithValue(launchService),
    iconExtractorProvider.overrideWithValue(
      iconExtractor ?? FakeIconExtractor(),
    ),
    iconsDirProvider.overrideWithValue(iconsDir),
    localeFileProvider.overrideWithValue(localeFile),
    themeFileProvider.overrideWithValue(themeFile),
    edgeWarningFileProvider.overrideWithValue(edgeWarningFile),
    hideTrayFileProvider.overrideWithValue(hideTrayFile),
    globalHotkeyFileProvider.overrideWithValue(globalHotkeyFile),
    appDataDirProvider.overrideWithValue(tempDir),
    packageInfoProvider.overrideWith(
      (ref) async => PackageInfo(
        appName: 'LinkUnbound',
        packageName: 'linkunbound',
        version: '1.0.0',
        buildNumber: '1',
      ),
    ),
    updateInfoProvider.overrideWith((ref) async => updateInfo),
    // The real exporter dumps the registry and opens an Explorer/Finder
    // window on the host machine.
    diagnosticsExporterProvider.overrideWithValue(
      diagnosticsExporter ??
          ({required Directory appDataDir, required String appVersion}) async =>
              '${appDataDir.path}${Platform.pathSeparator}fake-diag.zip',
    ),
  ];

  return (
    overrides: overrides,
    browserService: browserService,
    ruleService: ruleService,
    launchService: launchService,
    tempDir: tempDir,
  );
}
