import 'dart:io';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:package_info_plus/package_info_plus.dart';

import 'package:linkunbound/providers.dart';
import 'helpers.dart';

final class _CountingRegistrationService implements RegistrationService {
  _CountingRegistrationService({
    required this.isDefaultValue,
    required this.associations,
  });

  final bool isDefaultValue;
  final Set<String> associations;
  int isDefaultReads = 0;
  int defaultAssociationsReads = 0;

  @override
  Future<Set<String>> get defaultAssociations async {
    defaultAssociationsReads++;
    return associations;
  }

  @override
  Future<bool> get isDefault async {
    isDefaultReads++;
    return isDefaultValue;
  }

  @override
  Future<void> register(String executablePath) async {}

  @override
  Future<void> unregister() async {}
}

final class _CountingStartupService implements StartupService {
  _CountingStartupService(this.isEnabledValue);

  final bool isEnabledValue;
  int isEnabledReads = 0;

  @override
  Future<void> disable() async {}

  @override
  Future<void> enable(String executablePath) async {}

  @override
  Future<bool> get isEnabled async {
    isEnabledReads++;
    return isEnabledValue;
  }
}

void main() {
  group('override guards', () {
    test('browserServiceProvider requires an override', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      expect(() => container.read(browserServiceProvider), throwsStateError);
    });
  });

  group('EdgeWarningNotifier', () {
    late Directory tempDir;
    late File edgeWarningFile;

    setUp(() {
      tempDir = Directory.systemTemp.createTempSync('edge_warning_test_');
      edgeWarningFile = File('${tempDir.path}/edge_warning_dismissed');
    });

    tearDown(() {
      if (tempDir.existsSync()) {
        tempDir.deleteSync(recursive: true);
      }
    });

    ProviderContainer makeContainer() => ProviderContainer(
      overrides: [edgeWarningFileProvider.overrideWithValue(edgeWarningFile)],
    );

    test('build returns false when dismissal file is missing', () {
      final container = makeContainer();
      addTearDown(container.dispose);

      expect(container.read(edgeWarningDismissedProvider), isFalse);
    });

    test('build returns true when dismissal file exists', () {
      edgeWarningFile.writeAsStringSync('1');
      final container = makeContainer();
      addTearDown(container.dispose);

      expect(container.read(edgeWarningDismissedProvider), isTrue);
    });

    test('dismiss writes the marker file and updates state', () {
      final container = makeContainer();
      addTearDown(container.dispose);

      container.read(edgeWarningDismissedProvider.notifier).dismiss();

      expect(edgeWarningFile.existsSync(), isTrue);
      expect(edgeWarningFile.readAsStringSync(), '1');
      expect(container.read(edgeWarningDismissedProvider), isTrue);
    });
  });

  group('async providers', () {
    test(
      'isDefaultBrowserProvider resolves from registration service',
      () async {
        final registration = _CountingRegistrationService(
          isDefaultValue: true,
          associations: {'http', 'https'},
        );
        final container = ProviderContainer(
          overrides: [
            registrationServiceProvider.overrideWithValue(registration),
          ],
        );
        addTearDown(container.dispose);

        expect(await container.read(isDefaultBrowserProvider.future), isTrue);
        expect(registration.isDefaultReads, 1);
      },
    );

    test(
      'defaultAssociationsProvider resolves from registration service',
      () async {
        final registration = _CountingRegistrationService(
          isDefaultValue: false,
          associations: {'.html', 'http'},
        );
        final container = ProviderContainer(
          overrides: [
            registrationServiceProvider.overrideWithValue(registration),
          ],
        );
        addTearDown(container.dispose);

        expect(await container.read(defaultAssociationsProvider.future), {
          '.html',
          'http',
        });
        expect(registration.defaultAssociationsReads, 1);
      },
    );

    test('isStartupEnabledProvider resolves from startup service', () async {
      final startup = _CountingStartupService(true);
      final container = ProviderContainer(
        overrides: [startupServiceProvider.overrideWithValue(startup)],
      );
      addTearDown(container.dispose);

      expect(await container.read(isStartupEnabledProvider.future), isTrue);
      expect(startup.isEnabledReads, 1);
    });

    test('packageInfoProvider returns mocked package metadata', () async {
      PackageInfo.setMockInitialValues(
        appName: 'LinkUnbound',
        packageName: 'dev.rg.LinkUnbound',
        version: '9.9.9',
        buildNumber: '42',
        buildSignature: 'sig',
      );
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final info = await container.read(packageInfoProvider.future);

      expect(info.appName, 'LinkUnbound');
      expect(info.version, '9.9.9');
      expect(info.buildNumber, '42');
    });
  });

  group('updateInfoProvider TTL', () {
    late Directory tempDir;

    setUp(() {
      tempDir = Directory.systemTemp.createTempSync('update_ttl_test_');
    });

    tearDown(() {
      if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
    });

    test('override seam works — returns injected UpdateInfo', () async {
      const fakeInfo = UpdateInfo(
        latestVersion: '99.0.0',
        releaseUrl: 'https://example.com',
      );
      final fixtures = makeFixtures(dir: tempDir, updateInfo: fakeInfo);
      final container = ProviderContainer(overrides: fixtures.overrides);
      addTearDown(container.dispose);

      final result = await container.read(updateInfoProvider.future);
      expect(result?.latestVersion, '99.0.0');
    });

    test('override seam works — returns null when no update', () async {
      final fixtures = makeFixtures(dir: tempDir, updateInfo: null);
      final container = ProviderContainer(overrides: fixtures.overrides);
      addTearDown(container.dispose);

      final result = await container.read(updateInfoProvider.future);
      expect(result, isNull);
    });

    test('provider is invalidated and re-fetched after invalidateSelf', () async {
      var callCount = 0;
      final container = ProviderContainer(
        overrides: [
          updateInfoProvider.overrideWith((ref) async {
            callCount++;
            return null;
          }),
        ],
      );
      addTearDown(container.dispose);

      await container.read(updateInfoProvider.future);
      expect(callCount, 1);

      container.invalidate(updateInfoProvider);
      await container.read(updateInfoProvider.future);
      expect(callCount, 2);
    });
  });
}
