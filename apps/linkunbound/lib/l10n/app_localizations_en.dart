// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get exit => 'Exit';

  @override
  String get traySettings => 'Settings';

  @override
  String get copyUrl => 'Copy URL';

  @override
  String get alwaysOpenHere => 'Always open here';

  @override
  String get tabGeneral => 'General';

  @override
  String get tabRules => 'Rules';

  @override
  String get tabAbout => 'About';

  @override
  String get tabMaintenance => 'Maintenance';

  @override
  String get sectionDefaultBrowser => 'DEFAULT BROWSER';

  @override
  String get isDefaultBrowser => 'LinkUnbound is set as the default browser';

  @override
  String get notDefaultBrowser => 'LinkUnbound is not the default browser';

  @override
  String get setDefault => 'Set default';

  @override
  String get sectionStartup => 'STARTUP';

  @override
  String get launchAtStartup => 'Launch at system startup';

  @override
  String get startupManagedByWindows =>
      'Managed by Windows Settings > Startup Apps';

  @override
  String get sectionLanguage => 'LANGUAGE';

  @override
  String get languageAuto => 'Automatic (system)';

  @override
  String get languageEnglish => 'English';

  @override
  String get languageSpanish => 'Spanish';

  @override
  String get sectionAppearance => 'APPEARANCE';

  @override
  String get themeLabel => 'Theme';

  @override
  String get themeSystem => 'Automatic (system)';

  @override
  String get themeLight => 'Light';

  @override
  String get themeDark => 'Dark';

  @override
  String get sectionBrowsers => 'BROWSERS';

  @override
  String get addBrowserTooltip => 'Add custom browser';

  @override
  String get refreshBrowsersTooltip => 'Refresh browsers';

  @override
  String get menuEdit => 'Edit';

  @override
  String get menuDuplicate => 'Duplicate';

  @override
  String get menuRemove => 'Remove';

  @override
  String refreshResult(int added, int removed) {
    return '$added added, $removed removed';
  }

  @override
  String get refreshNoChanges => 'No changes detected';

  @override
  String get editBrowserTitle => 'Edit browser';

  @override
  String get addBrowserTitle => 'Add custom browser';

  @override
  String get fieldName => 'Name';

  @override
  String get fieldExecutablePath => 'Executable path';

  @override
  String get fieldExtraArgs => 'Extra arguments (space-separated)';

  @override
  String get fieldIconPath => 'Custom icon path (optional)';

  @override
  String get fieldIconHint => 'Leave empty to auto-detect from exe';

  @override
  String get cancel => 'Cancel';

  @override
  String get add => 'Add';

  @override
  String get save => 'Save';

  @override
  String get confirm => 'Confirm';

  @override
  String get sectionUrlRules => 'URL RULES';

  @override
  String get noRulesYet =>
      'No rules yet. Rules are created from the browser picker when you check \"Always open here\".';

  @override
  String get columnDomain => 'Domain';

  @override
  String get columnBrowser => 'Browser';

  @override
  String get deleteRuleTitle => 'Delete rule';

  @override
  String deleteRuleContent(String domain) {
    return 'Remove the rule for \"$domain\"?';
  }

  @override
  String get delete => 'Delete';

  @override
  String get deleteRuleTooltip => 'Delete rule';

  @override
  String get sectionAbout => 'ABOUT';

  @override
  String appVersion(String version) {
    return 'Version $version';
  }

  @override
  String get appDescription => 'Open-source browser picker for Windows.';

  @override
  String get mitLicense => 'GPL-3.0 License';

  @override
  String get resetConfigLabel => 'Reset configuration';

  @override
  String get resetConfigDescription =>
      'Clear all browsers and rules, then re-scan';

  @override
  String get unregisterLabel => 'Unregister LinkUnbound';

  @override
  String get unregisterDescription => 'Remove from Windows browser list';

  @override
  String get resetConfigTitle => 'Reset configuration';

  @override
  String get resetConfigContent =>
      'This will delete all browsers, rules and icons, then re-scan installed browsers. Continue?';

  @override
  String get reset => 'Reset';

  @override
  String get unregisterTitle => 'Unregister LinkUnbound';

  @override
  String get unregisterContent =>
      'This will remove LinkUnbound from the Windows browser list. You may need to change your default browser in Windows Settings afterwards. Continue?';

  @override
  String get unregisterAction => 'Unregister';

  @override
  String updateAvailable(String version) {
    return 'Version $version available';
  }

  @override
  String get updateDownload => 'Download';

  @override
  String updateAvailableStore(String version) {
    return 'Version $version available — check Microsoft Store for the new version';
  }

  @override
  String get updateTooltip =>
      'New version available — check for updates in Settings';

  @override
  String get sectionSupport => 'SUPPORT';

  @override
  String get donateLabel => 'Buy me a coffee';

  @override
  String get donateDescription =>
      'LinkUnbound is free and always will be. If it saves you time, consider supporting development.';

  @override
  String get sectionOtherTools => 'OTHER TOOLS';

  @override
  String get otherToolCopyPaste => 'CopyPaste';

  @override
  String get otherToolCopyPasteDescription =>
      'Free, open source clipboard manager for Windows, macOS and Linux. Same philosophy: no ads, no telemetry, everything local.';

  @override
  String get edgeWarningTitle => 'Microsoft Edge detected';

  @override
  String get edgeWarningBody =>
      'Microsoft Teams, Outlook, and other Microsoft 365 apps may open links directly in Edge, ignoring your default browser. This is a Microsoft design decision that LinkUnbound cannot override.';

  @override
  String get edgeWarningNote =>
      'You can change this behavior from each app\'s settings. Some organizations enforce Edge through group policies.';

  @override
  String get edgeWarningDismiss => 'Got it, don\'t show again';

  @override
  String get sectionMaintenance => 'MAINTENANCE';

  @override
  String get exportDiagnosticsLabel => 'Export diagnostics';

  @override
  String get exportDiagnosticsDescription =>
      'Generate a ZIP with system info, registry data, and logs for troubleshooting';

  @override
  String get errorStartupToggle => 'Could not change startup setting';

  @override
  String get errorUnregister => 'Could not unregister LinkUnbound';

  @override
  String get errorExportDiagnostics => 'Could not export diagnostics';

  @override
  String get errorResetConfig => 'Could not reset configuration';

  @override
  String get sectionAccessibility => 'ACCESSIBILITY';

  @override
  String get globalHotkeyLabel => 'Global shortcut to open settings';

  @override
  String get globalHotkeyNone => 'None (disabled)';

  @override
  String get hideTrayLabel => 'Hide tray / menu bar icon';

  @override
  String get hideTraySubtitleNoHotkey =>
      'Requires a global shortcut to be configured first';

  @override
  String get hideTraySubtitleMac =>
      'You can also reach the app by relaunching it from Applications';

  @override
  String get hideTraySubtitleWindows =>
      'You can reach the app via its global shortcut';

  @override
  String get pickerNoBrowsers =>
      'No browsers detected. Open Settings to add one.';

  @override
  String get edgeProtocolLabel => 'Capture links from Microsoft apps';

  @override
  String get edgeProtocolDescription =>
      'Teams, Outlook and Start menu search open links through Edge directly, bypassing the default browser. Enable this to let LinkUnbound offer a choice for those links too.';

  @override
  String get pickerPrivateHint => 'Shift = private';

  @override
  String alwaysOpenFromApp(String app) {
    return 'Always open links from $app here';
  }

  @override
  String get diagnosticsTitle => 'Link capture problem detected';

  @override
  String get diagnosticsStaleHandler =>
      'The registered handler points somewhere else, so links will not reach LinkUnbound.';

  @override
  String get diagnosticsDevBuild =>
      'Running from a local build, which never registers itself. Install the app to capture links.';

  @override
  String get diagnosticsRepair => 'Repair';

  @override
  String get diagnosticsRepaired => 'Registration repaired.';

  @override
  String get diagnosticsRepairFailed => 'Could not repair the registration.';

  @override
  String ruleFromApp(String app) {
    return 'Links from $app';
  }
}
