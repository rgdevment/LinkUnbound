import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:window_manager/window_manager.dart';

import 'l10n/app_localizations.dart';
import 'providers.dart';
import 'ui/picker/picker_window.dart';
import 'ui/settings/settings_window.dart';
import 'ui/shared/app_theme.dart';

final class NavigateApp extends ConsumerStatefulWidget {
  const NavigateApp({super.key});

  @override
  ConsumerState<NavigateApp> createState() => _NavigateAppState();
}

final class _NavigateAppState extends ConsumerState<NavigateApp>
    with WindowListener {
  // Guards against blur events fired before the picker has settled.
  // Set to true either on the first onWindowFocus after showing, or after a
  // fallback timer — focus events are unreliable on Windows when the window
  // is shown programmatically without foreground rights, so we keep the timer.
  bool _pickerBlurReady = false;
  Timer? _blurGuardTimer;

  // Debounce: skip provider invalidation if we invalidated less than 2s ago.
  DateTime? _lastFocusInvalidation;

  @override
  void initState() {
    super.initState();
    windowManager.addListener(this);
  }

  @override
  void dispose() {
    _blurGuardTimer?.cancel();
    windowManager.removeListener(this);
    super.dispose();
  }

  @override
  void onWindowClose() async {
    await windowManager.hide();
    ref.read(appStateProvider.notifier).hide();
  }

  @override
  void onWindowFocus() {
    final now = DateTime.now();
    if (_lastFocusInvalidation == null ||
        now.difference(_lastFocusInvalidation!) >= const Duration(seconds: 2)) {
      _lastFocusInvalidation = now;
      ref.invalidate(isDefaultBrowserProvider);
      ref.invalidate(isStartupEnabledProvider);
    }

    final mode = ref.read(appStateProvider).mode;
    if (mode == AppMode.picker && !_pickerBlurReady) {
      _blurGuardTimer?.cancel();
      _pickerBlurReady = true;
    }
  }

  @override
  void onWindowBlur() {
    final mode = ref.read(appStateProvider).mode;
    if (mode != AppMode.picker) return;
    if (!_pickerBlurReady) return;
    ref.read(appStateProvider.notifier).hide();
  }

  @override
  Widget build(BuildContext context) {
    final appState = ref.watch(appStateProvider);
    final locale = ref.watch(localeProvider);

    ref.listen<AppState>(appStateProvider, (prev, next) {
      if (prev?.mode == next.mode) return;
      _blurGuardTimer?.cancel();
      _pickerBlurReady = false;

      if (next.mode == AppMode.picker) {
        // Fallback: if focus events never fire (e.g. on Windows without
        // foreground rights), mark ready after 350 ms so blur can still close.
        _blurGuardTimer = Timer(const Duration(milliseconds: 350), () {
          _pickerBlurReady = true;
        });
      }

      if (next.mode == AppMode.hidden) return;
      WidgetsBinding.instance.addPostFrameCallback((_) async {
        await windowManager.show();
        await windowManager.focus();
      });
    });

    return MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: AppTheme.light,
      darkTheme: AppTheme.dark,
      themeMode: ThemeMode.system,
      locale: locale,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: switch (appState.mode) {
        AppMode.hidden => const ColoredBox(color: Color(0xFF1E1E2E)),
        AppMode.settings => const SettingsWindow(),
        AppMode.picker => PickerWindow(url: appState.pendingUrl ?? ''),
      },
    );
  }
}
