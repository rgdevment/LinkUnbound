import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linkunbound_core/linkunbound_core.dart';

import 'package:linkunbound/providers.dart';

import 'helpers.dart';

void main() {
  group('AppStateNotifier', () {
    ProviderContainer makeContainer() => ProviderContainer();

    test('initial state is hidden with no pending url', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(c.read(appStateProvider).mode, AppMode.hidden);
      expect(c.read(appStateProvider).pendingUrl, isNull);
    });

    test('showSettings sets mode to settings', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(appStateProvider.notifier).showSettings();
      expect(c.read(appStateProvider).mode, AppMode.settings);
      expect(c.read(appStateProvider).pendingUrl, isNull);
    });

    test('showPicker sets mode to picker and stores url', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(appStateProvider.notifier).showPicker('https://example.com');
      expect(c.read(appStateProvider).mode, AppMode.picker);
      expect(c.read(appStateProvider).pendingUrl, 'https://example.com');
    });

    test('hide resets to hidden and clears url', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(appStateProvider.notifier).showPicker('https://example.com');
      c.read(appStateProvider.notifier).hide();
      expect(c.read(appStateProvider).mode, AppMode.hidden);
      expect(c.read(appStateProvider).pendingUrl, isNull);
    });

    test('showSettings after picker resets pendingUrl', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(appStateProvider.notifier).showPicker('https://example.com');
      c.read(appStateProvider.notifier).showSettings();
      expect(c.read(appStateProvider).mode, AppMode.settings);
      expect(c.read(appStateProvider).pendingUrl, isNull);
    });
  });

  group('LocaleNotifier', () {
    late Directory tempDir;
    late File localeFile;

    setUp(() {
      tempDir = Directory.systemTemp.createTempSync('locale_notifier_test_');
      localeFile = File('${tempDir.path}/locale');
    });

    tearDown(() {
      if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
    });

    ProviderContainer makeContainer() => ProviderContainer(
      overrides: [localeFileProvider.overrideWithValue(localeFile)],
    );

    test('returns null when locale file does not exist', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(c.read(localeProvider), isNull);
    });

    test('returns Locale en when file contains en', () {
      localeFile.writeAsStringSync('en');
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(c.read(localeProvider)?.languageCode, 'en');
    });

    test('returns Locale es when file contains es', () {
      localeFile.writeAsStringSync('es');
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(c.read(localeProvider)?.languageCode, 'es');
    });

    test('returns null for unrecognized language code', () {
      localeFile.writeAsStringSync('de');
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(c.read(localeProvider), isNull);
    });

    test('returns null for whitespace-only file', () {
      localeFile.writeAsStringSync('   ');
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(c.read(localeProvider), isNull);
    });

    test('setLocale writes languageCode to file', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(localeProvider.notifier).setLocale(const Locale('es'));
      expect(localeFile.readAsStringSync(), 'es');
    });

    test('setLocale updates state', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(localeProvider.notifier).setLocale(const Locale('en'));
      expect(c.read(localeProvider)?.languageCode, 'en');
    });

    test('setLocale null deletes file', () {
      localeFile.writeAsStringSync('en');
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(localeProvider.notifier).setLocale(null);
      expect(localeFile.existsSync(), isFalse);
    });

    test('setLocale null when file absent does not throw', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(
        () => c.read(localeProvider.notifier).setLocale(null),
        returnsNormally,
      );
    });

    test('setLocale null sets state to null', () {
      localeFile.writeAsStringSync('en');
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(localeProvider.notifier).setLocale(null);
      expect(c.read(localeProvider), isNull);
    });
  });

  group('ThemeModeNotifier', () {
    late Directory tempDir;
    late File themeFile;

    setUp(() {
      tempDir = Directory.systemTemp.createTempSync('theme_notifier_test_');
      themeFile = File('${tempDir.path}/theme');
    });

    tearDown(() {
      if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
    });

    ProviderContainer makeContainer() => ProviderContainer(
      overrides: [themeFileProvider.overrideWithValue(themeFile)],
    );

    test('returns system when theme file does not exist', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(c.read(themeModeProvider), ThemeMode.system);
    });

    test('returns light when file contains light', () {
      themeFile.writeAsStringSync('light');
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(c.read(themeModeProvider), ThemeMode.light);
    });

    test('returns dark when file contains dark', () {
      themeFile.writeAsStringSync('dark');
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(c.read(themeModeProvider), ThemeMode.dark);
    });

    test('returns system for unrecognized content', () {
      themeFile.writeAsStringSync('sepia');
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(c.read(themeModeProvider), ThemeMode.system);
    });

    test('setThemeMode dark writes file and updates state', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(themeModeProvider.notifier).setThemeMode(ThemeMode.dark);
      expect(themeFile.readAsStringSync(), 'dark');
      expect(c.read(themeModeProvider), ThemeMode.dark);
    });

    test('setThemeMode light writes file and updates state', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(themeModeProvider.notifier).setThemeMode(ThemeMode.light);
      expect(themeFile.readAsStringSync(), 'light');
      expect(c.read(themeModeProvider), ThemeMode.light);
    });

    test('setThemeMode system deletes file and updates state', () {
      themeFile.writeAsStringSync('dark');
      final c = makeContainer();
      addTearDown(c.dispose);
      c.read(themeModeProvider.notifier).setThemeMode(ThemeMode.system);
      expect(themeFile.existsSync(), isFalse);
      expect(c.read(themeModeProvider), ThemeMode.system);
    });

    test('setThemeMode system when file absent does not throw', () {
      final c = makeContainer();
      addTearDown(c.dispose);
      expect(
        () => c.read(themeModeProvider.notifier).setThemeMode(ThemeMode.system),
        returnsNormally,
      );
    });
  });

  group('BrowsersNotifier', () {
    late Directory tempDir;

    setUp(() {
      tempDir = Directory.systemTemp.createTempSync('browsers_notifier_test_');
    });

    tearDown(() {
      if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
    });

    BrowserService makeService({
      List<Browser> browsers = const [],
      List<Browser> detected = const [],
    }) {
      final service = BrowserService(
        configFile: File('${tempDir.path}/browsers.json'),
        browserDetector: FakeBrowserDetector(detected),
      );
      for (final b in browsers) {
        service.addBrowser(b);
      }
      return service;
    }

    ProviderContainer makeContainer(BrowserService service) =>
        ProviderContainer(
          overrides: [browserServiceProvider.overrideWithValue(service)],
        );

    test('initial state reads browsers from service', () {
      const chrome = Browser(
        id: 'chrome',
        name: 'Chrome',
        executablePath: 'chrome.exe',
        iconPath: 'chrome.png',
      );
      final c = makeContainer(makeService(browsers: [chrome]));
      addTearDown(c.dispose);
      expect(c.read(browsersProvider), hasLength(1));
      expect(c.read(browsersProvider).first.id, 'chrome');
    });

    test('initial state is empty when service has no browsers', () {
      final c = makeContainer(makeService());
      addTearDown(c.dispose);
      expect(c.read(browsersProvider), isEmpty);
    });

    test('add appends a browser', () async {
      final c = makeContainer(makeService());
      addTearDown(c.dispose);
      await c
          .read(browsersProvider.notifier)
          .add(
            const Browser(
              id: 'ff',
              name: 'Firefox',
              executablePath: 'ff.exe',
              iconPath: 'ff.png',
            ),
          );
      expect(c.read(browsersProvider), hasLength(1));
      expect(c.read(browsersProvider).first.id, 'ff');
    });

    test('remove deletes by id', () async {
      final service = makeService(
        browsers: [
          const Browser(
            id: 'x',
            name: 'X',
            executablePath: 'x.exe',
            iconPath: 'x.png',
          ),
        ],
      );
      final c = makeContainer(service);
      addTearDown(c.dispose);
      await c.read(browsersProvider.notifier).remove('x');
      expect(c.read(browsersProvider), isEmpty);
    });

    test('update replaces browser by id', () async {
      final service = makeService(
        browsers: [
          const Browser(
            id: 'x',
            name: 'X',
            executablePath: 'x.exe',
            iconPath: 'x.png',
          ),
        ],
      );
      final c = makeContainer(service);
      addTearDown(c.dispose);
      await c
          .read(browsersProvider.notifier)
          .update(
            'x',
            const Browser(
              id: 'x',
              name: 'Updated',
              executablePath: 'x.exe',
              iconPath: 'x.png',
            ),
          );
      expect(c.read(browsersProvider).first.name, 'Updated');
    });

    test('reorder changes browser order', () async {
      final service = makeService(
        browsers: [
          const Browser(
            id: 'a',
            name: 'A',
            executablePath: 'a.exe',
            iconPath: 'a.png',
          ),
          const Browser(
            id: 'b',
            name: 'B',
            executablePath: 'b.exe',
            iconPath: 'b.png',
          ),
          const Browser(
            id: 'c',
            name: 'C',
            executablePath: 'c.exe',
            iconPath: 'c.png',
          ),
        ],
      );
      final c = makeContainer(service);
      addTearDown(c.dispose);
      await c.read(browsersProvider.notifier).reorder(0, 2);
      expect(c.read(browsersProvider).map((b) => b.id).toList(), [
        'b',
        'c',
        'a',
      ]);
    });

    test('refresh keeps custom browsers and picks up detected ones', () async {
      const custom = Browser(
        id: 'my',
        name: 'My',
        executablePath: 'my.exe',
        iconPath: 'my.png',
        isCustom: true,
      );
      const detected = Browser(
        id: 'ff',
        name: 'Firefox',
        executablePath: 'ff.exe',
        iconPath: 'ff.png',
      );
      final service = makeService(browsers: [custom], detected: [detected]);
      final c = makeContainer(service);
      addTearDown(c.dispose);
      await c.read(browsersProvider.notifier).refresh();
      final ids = c.read(browsersProvider).map((b) => b.id).toList();
      expect(ids, contains('my'));
      expect(ids, contains('ff'));
    });

    test('refresh removes non-custom browser no longer detected', () async {
      const old = Browser(
        id: 'old',
        name: 'Old',
        executablePath: 'old.exe',
        iconPath: 'old.png',
      );
      final service = makeService(browsers: [old], detected: []);
      final c = makeContainer(service);
      addTearDown(c.dispose);
      await c.read(browsersProvider.notifier).refresh();
      expect(c.read(browsersProvider), isEmpty);
    });
  });

  group('RulesNotifier', () {
    late Directory tempDir;

    setUp(() {
      tempDir = Directory.systemTemp.createTempSync('rules_notifier_test_');
    });

    tearDown(() {
      if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
    });

    RuleService makeService({List<Rule> rules = const []}) {
      final service = RuleService(
        rulesFile: File('${tempDir.path}/rules.json'),
      );
      for (final r in rules) {
        service.addRule(r);
      }
      return service;
    }

    ProviderContainer makeContainer(RuleService service) => ProviderContainer(
      overrides: [ruleServiceProvider.overrideWithValue(service)],
    );

    test('initial state reads rules from service', () {
      final c = makeContainer(
        makeService(
          rules: [const Rule(domain: 'a.com', browserId: 'chrome')],
        ),
      );
      addTearDown(c.dispose);
      expect(c.read(rulesProvider), hasLength(1));
    });

    test('initial state is empty when service has no rules', () {
      final c = makeContainer(makeService());
      addTearDown(c.dispose);
      expect(c.read(rulesProvider), isEmpty);
    });

    test('updateRule changes browserId', () async {
      final c = makeContainer(
        makeService(
          rules: [const Rule(domain: 'a.com', browserId: 'chrome')],
        ),
      );
      addTearDown(c.dispose);
      await c
          .read(rulesProvider.notifier)
          .updateRule('a.com', browserId: 'firefox');
      expect(c.read(rulesProvider).first.browserId, 'firefox');
    });

    test('removeRule deletes by domain', () async {
      final c = makeContainer(
        makeService(
          rules: [const Rule(domain: 'a.com', browserId: 'chrome')],
        ),
      );
      addTearDown(c.dispose);
      await c.read(rulesProvider.notifier).removeRule('a.com');
      expect(c.read(rulesProvider), isEmpty);
    });

    test('updateRule persists state after multiple updates', () async {
      final c = makeContainer(
        makeService(
          rules: [
            const Rule(domain: 'a.com', browserId: 'chrome'),
            const Rule(domain: 'b.com', browserId: 'edge'),
          ],
        ),
      );
      addTearDown(c.dispose);
      await c
          .read(rulesProvider.notifier)
          .updateRule('a.com', browserId: 'firefox');
      expect(c.read(rulesProvider), hasLength(2));
      expect(
        c.read(rulesProvider).firstWhere((r) => r.domain == 'a.com').browserId,
        'firefox',
      );
      expect(
        c.read(rulesProvider).firstWhere((r) => r.domain == 'b.com').browserId,
        'edge',
      );
    });
  });
}
