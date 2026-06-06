import 'dart:io';
import 'dart:ui';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linkunbound_core/linkunbound_core.dart';
import 'package:package_info_plus/package_info_plus.dart';

import 'platform/macos/mac_diagnostics_service.dart';
import 'platform/windows/win_diagnostics_service.dart';

StateError _mustOverride() => StateError('Override at startup');

final browserServiceProvider = Provider<BrowserService>(
  (_) => throw _mustOverride(),
);

final ruleServiceProvider = Provider<RuleService>((_) => throw _mustOverride());

final registrationServiceProvider = Provider<RegistrationService>(
  (_) => throw _mustOverride(),
);

final startupServiceProvider = Provider<StartupService>(
  (_) => throw _mustOverride(),
);

final iconExtractorProvider = Provider<IconExtractor>(
  (_) => throw _mustOverride(),
);

final iconsDirProvider = Provider<Directory>((_) => throw _mustOverride());

final launchServiceProvider = Provider<LaunchService>(
  (_) => throw _mustOverride(),
);

final localeFileProvider = Provider<File>((_) => throw _mustOverride());

final edgeWarningFileProvider = Provider<File>((_) => throw _mustOverride());

final hideTrayFileProvider = Provider<File>((_) => throw _mustOverride());

final globalHotkeyFileProvider = Provider<File>((_) => throw _mustOverride());

final appDataDirProvider = Provider<Directory>((_) => throw _mustOverride());

typedef DiagnosticsExporter =
    Future<String> Function({
      required Directory appDataDir,
      required String appVersion,
    });

/// Injectable so widget tests never run the real exporter, which dumps the
/// registry and opens an Explorer/Finder window on the host.
final diagnosticsExporterProvider = Provider<DiagnosticsExporter>(
  (_) => Platform.isMacOS ? exportMacDiagnostics : exportDiagnostics,
);

/// Async callback that releases platform resources and terminates the process.
/// Overridden at startup with `bindings.release()` + `exit(0)`.
typedef ExitAppCallback = Future<void> Function();
final exitAppProvider = Provider<ExitAppCallback>((_) => throw _mustOverride());

final edgeWarningDismissedProvider =
    NotifierProvider<EdgeWarningNotifier, bool>(EdgeWarningNotifier.new);

final class EdgeWarningNotifier extends Notifier<bool> {
  @override
  bool build() {
    final file = ref.read(edgeWarningFileProvider);
    return file.existsSync();
  }

  void dismiss() {
    final file = ref.read(edgeWarningFileProvider);
    file.writeAsStringSync('1');
    state = true;
  }
}

final localeProvider = NotifierProvider<LocaleNotifier, Locale?>(
  LocaleNotifier.new,
);

final class LocaleNotifier extends Notifier<Locale?> {
  @override
  Locale? build() {
    final file = ref.read(localeFileProvider);
    if (!file.existsSync()) return null;
    final code = file.readAsStringSync().trim();
    if (code == 'en' || code == 'es') return Locale(code);
    return null;
  }

  void setLocale(Locale? locale) {
    final file = ref.read(localeFileProvider);
    if (locale == null) {
      if (file.existsSync()) file.deleteSync();
    } else {
      file.writeAsStringSync(locale.languageCode);
    }
    state = locale;
  }
}

enum AppMode { hidden, settings, picker }

final class AppState {
  AppState({this.mode = AppMode.hidden, this.pendingUrl});
  final AppMode mode;
  final String? pendingUrl;
}

final appStateProvider = NotifierProvider<AppStateNotifier, AppState>(
  AppStateNotifier.new,
);

final class AppStateNotifier extends Notifier<AppState> {
  @override
  AppState build() => AppState();

  void showSettings() => state = AppState(mode: AppMode.settings);

  void showPicker(String url) =>
      state = AppState(mode: AppMode.picker, pendingUrl: url);

  void hide() => state = AppState();
}

final browsersProvider = NotifierProvider<BrowsersNotifier, List<Browser>>(
  BrowsersNotifier.new,
);

final class BrowsersNotifier extends Notifier<List<Browser>> {
  @override
  List<Browser> build() => ref.read(browserServiceProvider).browsers;

  Future<({int added, int removed})> refresh() async {
    final service = ref.read(browserServiceProvider);
    final result = await service.scanAndMerge();
    state = service.browsers;
    return result;
  }

  Future<void> add(Browser browser) async {
    final service = ref.read(browserServiceProvider);
    service.addBrowser(browser);
    await service.save();
    state = service.browsers;
  }

  Future<void> remove(String id) async {
    final service = ref.read(browserServiceProvider);
    service.removeBrowser(id);
    await service.save();
    state = service.browsers;
  }

  Future<void> update(String id, Browser browser) async {
    final service = ref.read(browserServiceProvider);
    service.updateBrowser(id, browser);
    await service.save();
    state = service.browsers;
  }

  Future<void> reorder(int oldIndex, int newIndex) async {
    final service = ref.read(browserServiceProvider);
    service.reorder(oldIndex, newIndex);
    await service.save();
    state = service.browsers;
  }
}

final rulesProvider = NotifierProvider<RulesNotifier, List<Rule>>(
  RulesNotifier.new,
);

final class RulesNotifier extends Notifier<List<Rule>> {
  @override
  List<Rule> build() => ref.read(ruleServiceProvider).rules;

  Future<void> updateRule(String domain, {required String browserId}) async {
    final service = ref.read(ruleServiceProvider);
    service.updateRule(domain, browserId: browserId);
    await service.save();
    state = service.rules;
  }

  Future<void> removeRule(String domain) async {
    final service = ref.read(ruleServiceProvider);
    service.removeRule(domain);
    await service.save();
    state = service.rules;
  }
}

final isDefaultBrowserProvider = FutureProvider.autoDispose<bool>((ref) {
  return ref.read(registrationServiceProvider).isDefault;
});

final defaultAssociationsProvider = FutureProvider.autoDispose<Set<String>>((
  ref,
) {
  return ref.read(registrationServiceProvider).defaultAssociations;
});

final isStartupEnabledProvider = FutureProvider.autoDispose<bool>((ref) {
  return ref.read(startupServiceProvider).isEnabled;
});

final packageInfoProvider = FutureProvider<PackageInfo>((ref) {
  return PackageInfo.fromPlatform();
});

const _updateService = UpdateService(owner: 'rgdevment', repo: 'LinkUnbound');

/// Timestamp of the last completed update check. Module-level so it survives
/// provider invalidations within the same process lifetime.
DateTime? _lastUpdateFetch;

/// Cached result of the last check, returned within the TTL window to avoid
/// redundant network calls.
UpdateInfo? _cachedUpdateInfo;

/// After this duration the cached result is stale and the next provider
/// evaluation re-fetches from GitHub.
const _updateTtl = Duration(hours: 6);

final updateInfoProvider = FutureProvider<UpdateInfo?>((ref) async {
  final now = DateTime.now();
  final lastFetch = _lastUpdateFetch;

  // Within the TTL window, return the cached result without a network call.
  if (lastFetch != null && now.difference(lastFetch) < _updateTtl) {
    return _cachedUpdateInfo;
  }

  _lastUpdateFetch = now;
  final info = await ref.watch(packageInfoProvider.future);
  _cachedUpdateInfo = await _updateService.checkForUpdate(info.version);
  return _cachedUpdateInfo;
});

final hideTrayProvider = NotifierProvider<HideTrayNotifier, bool>(
  HideTrayNotifier.new,
);

final class HideTrayNotifier extends Notifier<bool> {
  @override
  bool build() {
    final file = ref.read(hideTrayFileProvider);
    return file.existsSync();
  }

  void setHideTray(bool value) {
    final file = ref.read(hideTrayFileProvider);
    if (value) {
      file.writeAsStringSync('1');
    } else {
      if (file.existsSync()) file.deleteSync();
    }
    state = value;
  }
}

/// Serialised as a single line `modifiers+keyLabel`, e.g. "meta+alt+space".
/// Returns null when no hotkey is configured (file absent or blank).
final globalHotkeyProvider = NotifierProvider<GlobalHotkeyNotifier, String?>(
  GlobalHotkeyNotifier.new,
);

final class GlobalHotkeyNotifier extends Notifier<String?> {
  @override
  String? build() {
    final file = ref.read(globalHotkeyFileProvider);
    if (!file.existsSync()) return null;
    final raw = file.readAsStringSync().trim();
    return raw.isEmpty ? null : raw;
  }

  void setHotkey(String? serialized) {
    final file = ref.read(globalHotkeyFileProvider);
    if (serialized == null || serialized.isEmpty) {
      if (file.existsSync()) file.deleteSync();
      state = null;
    } else {
      file.writeAsStringSync(serialized);
      state = serialized;
    }
  }
}
