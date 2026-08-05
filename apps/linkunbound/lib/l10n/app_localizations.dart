import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_es.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations? of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations);
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('es'),
  ];

  /// No description provided for @exit.
  ///
  /// In en, this message translates to:
  /// **'Exit'**
  String get exit;

  /// No description provided for @traySettings.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get traySettings;

  /// No description provided for @copyUrl.
  ///
  /// In en, this message translates to:
  /// **'Copy URL'**
  String get copyUrl;

  /// No description provided for @alwaysOpenHere.
  ///
  /// In en, this message translates to:
  /// **'Always open here'**
  String get alwaysOpenHere;

  /// No description provided for @tabGeneral.
  ///
  /// In en, this message translates to:
  /// **'General'**
  String get tabGeneral;

  /// No description provided for @tabRules.
  ///
  /// In en, this message translates to:
  /// **'Rules'**
  String get tabRules;

  /// No description provided for @tabAbout.
  ///
  /// In en, this message translates to:
  /// **'About'**
  String get tabAbout;

  /// No description provided for @tabMaintenance.
  ///
  /// In en, this message translates to:
  /// **'Maintenance'**
  String get tabMaintenance;

  /// No description provided for @sectionDefaultBrowser.
  ///
  /// In en, this message translates to:
  /// **'DEFAULT BROWSER'**
  String get sectionDefaultBrowser;

  /// No description provided for @isDefaultBrowser.
  ///
  /// In en, this message translates to:
  /// **'LinkUnbound is set as the default browser'**
  String get isDefaultBrowser;

  /// No description provided for @notDefaultBrowser.
  ///
  /// In en, this message translates to:
  /// **'LinkUnbound is not the default browser'**
  String get notDefaultBrowser;

  /// No description provided for @setDefault.
  ///
  /// In en, this message translates to:
  /// **'Set default'**
  String get setDefault;

  /// No description provided for @sectionStartup.
  ///
  /// In en, this message translates to:
  /// **'STARTUP'**
  String get sectionStartup;

  /// No description provided for @launchAtStartup.
  ///
  /// In en, this message translates to:
  /// **'Launch at system startup'**
  String get launchAtStartup;

  /// Tooltip shown when the startup toggle is disabled because the app runs as MSIX and Windows owns that preference.
  ///
  /// In en, this message translates to:
  /// **'Managed by Windows Settings > Startup Apps'**
  String get startupManagedByWindows;

  /// No description provided for @sectionLanguage.
  ///
  /// In en, this message translates to:
  /// **'LANGUAGE'**
  String get sectionLanguage;

  /// No description provided for @languageAuto.
  ///
  /// In en, this message translates to:
  /// **'Automatic (system)'**
  String get languageAuto;

  /// No description provided for @languageEnglish.
  ///
  /// In en, this message translates to:
  /// **'English'**
  String get languageEnglish;

  /// No description provided for @languageSpanish.
  ///
  /// In en, this message translates to:
  /// **'Spanish'**
  String get languageSpanish;

  /// No description provided for @sectionAppearance.
  ///
  /// In en, this message translates to:
  /// **'APPEARANCE'**
  String get sectionAppearance;

  /// No description provided for @themeLabel.
  ///
  /// In en, this message translates to:
  /// **'Theme'**
  String get themeLabel;

  /// No description provided for @themeSystem.
  ///
  /// In en, this message translates to:
  /// **'Automatic (system)'**
  String get themeSystem;

  /// No description provided for @themeLight.
  ///
  /// In en, this message translates to:
  /// **'Light'**
  String get themeLight;

  /// No description provided for @themeDark.
  ///
  /// In en, this message translates to:
  /// **'Dark'**
  String get themeDark;

  /// No description provided for @sectionBrowsers.
  ///
  /// In en, this message translates to:
  /// **'BROWSERS'**
  String get sectionBrowsers;

  /// No description provided for @addBrowserTooltip.
  ///
  /// In en, this message translates to:
  /// **'Add custom browser'**
  String get addBrowserTooltip;

  /// No description provided for @refreshBrowsersTooltip.
  ///
  /// In en, this message translates to:
  /// **'Refresh browsers'**
  String get refreshBrowsersTooltip;

  /// No description provided for @menuEdit.
  ///
  /// In en, this message translates to:
  /// **'Edit'**
  String get menuEdit;

  /// No description provided for @menuDuplicate.
  ///
  /// In en, this message translates to:
  /// **'Duplicate'**
  String get menuDuplicate;

  /// No description provided for @menuRemove.
  ///
  /// In en, this message translates to:
  /// **'Remove'**
  String get menuRemove;

  /// No description provided for @refreshResult.
  ///
  /// In en, this message translates to:
  /// **'{added} added, {removed} removed'**
  String refreshResult(int added, int removed);

  /// No description provided for @refreshNoChanges.
  ///
  /// In en, this message translates to:
  /// **'No changes detected'**
  String get refreshNoChanges;

  /// No description provided for @editBrowserTitle.
  ///
  /// In en, this message translates to:
  /// **'Edit browser'**
  String get editBrowserTitle;

  /// No description provided for @addBrowserTitle.
  ///
  /// In en, this message translates to:
  /// **'Add custom browser'**
  String get addBrowserTitle;

  /// No description provided for @fieldName.
  ///
  /// In en, this message translates to:
  /// **'Name'**
  String get fieldName;

  /// No description provided for @fieldExecutablePath.
  ///
  /// In en, this message translates to:
  /// **'Executable path'**
  String get fieldExecutablePath;

  /// No description provided for @fieldExtraArgs.
  ///
  /// In en, this message translates to:
  /// **'Extra arguments (space-separated)'**
  String get fieldExtraArgs;

  /// No description provided for @fieldIconPath.
  ///
  /// In en, this message translates to:
  /// **'Custom icon path (optional)'**
  String get fieldIconPath;

  /// No description provided for @fieldIconHint.
  ///
  /// In en, this message translates to:
  /// **'Leave empty to auto-detect from exe'**
  String get fieldIconHint;

  /// No description provided for @cancel.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get cancel;

  /// No description provided for @add.
  ///
  /// In en, this message translates to:
  /// **'Add'**
  String get add;

  /// No description provided for @save.
  ///
  /// In en, this message translates to:
  /// **'Save'**
  String get save;

  /// No description provided for @confirm.
  ///
  /// In en, this message translates to:
  /// **'Confirm'**
  String get confirm;

  /// No description provided for @sectionUrlRules.
  ///
  /// In en, this message translates to:
  /// **'URL RULES'**
  String get sectionUrlRules;

  /// No description provided for @noRulesYet.
  ///
  /// In en, this message translates to:
  /// **'No rules yet. Rules are created from the browser picker when you check \"Always open here\".'**
  String get noRulesYet;

  /// No description provided for @columnDomain.
  ///
  /// In en, this message translates to:
  /// **'Domain'**
  String get columnDomain;

  /// No description provided for @columnBrowser.
  ///
  /// In en, this message translates to:
  /// **'Browser'**
  String get columnBrowser;

  /// No description provided for @deleteRuleTitle.
  ///
  /// In en, this message translates to:
  /// **'Delete rule'**
  String get deleteRuleTitle;

  /// No description provided for @deleteRuleContent.
  ///
  /// In en, this message translates to:
  /// **'Remove the rule for \"{domain}\"?'**
  String deleteRuleContent(String domain);

  /// No description provided for @delete.
  ///
  /// In en, this message translates to:
  /// **'Delete'**
  String get delete;

  /// No description provided for @deleteRuleTooltip.
  ///
  /// In en, this message translates to:
  /// **'Delete rule'**
  String get deleteRuleTooltip;

  /// No description provided for @sectionAbout.
  ///
  /// In en, this message translates to:
  /// **'ABOUT'**
  String get sectionAbout;

  /// No description provided for @appVersion.
  ///
  /// In en, this message translates to:
  /// **'Version {version}'**
  String appVersion(String version);

  /// No description provided for @appDescription.
  ///
  /// In en, this message translates to:
  /// **'Open-source browser picker for Windows and macOS.'**
  String get appDescription;

  /// No description provided for @licenseName.
  ///
  /// In en, this message translates to:
  /// **'GPL-3.0 License'**
  String get licenseName;

  /// No description provided for @resetConfigLabel.
  ///
  /// In en, this message translates to:
  /// **'Reset configuration'**
  String get resetConfigLabel;

  /// No description provided for @resetConfigDescription.
  ///
  /// In en, this message translates to:
  /// **'Clear all browsers and rules, then re-scan'**
  String get resetConfigDescription;

  /// No description provided for @unregisterLabel.
  ///
  /// In en, this message translates to:
  /// **'Unregister LinkUnbound'**
  String get unregisterLabel;

  /// No description provided for @unregisterDescription.
  ///
  /// In en, this message translates to:
  /// **'Remove from Windows browser list'**
  String get unregisterDescription;

  /// No description provided for @resetConfigTitle.
  ///
  /// In en, this message translates to:
  /// **'Reset configuration'**
  String get resetConfigTitle;

  /// No description provided for @resetConfigContent.
  ///
  /// In en, this message translates to:
  /// **'This will delete all browsers, rules and icons, then re-scan installed browsers. Continue?'**
  String get resetConfigContent;

  /// No description provided for @reset.
  ///
  /// In en, this message translates to:
  /// **'Reset'**
  String get reset;

  /// No description provided for @unregisterTitle.
  ///
  /// In en, this message translates to:
  /// **'Unregister LinkUnbound'**
  String get unregisterTitle;

  /// No description provided for @unregisterContent.
  ///
  /// In en, this message translates to:
  /// **'This will remove LinkUnbound from the Windows browser list. You may need to change your default browser in Windows Settings afterwards. Continue?'**
  String get unregisterContent;

  /// No description provided for @unregisterAction.
  ///
  /// In en, this message translates to:
  /// **'Unregister'**
  String get unregisterAction;

  /// No description provided for @updateAvailable.
  ///
  /// In en, this message translates to:
  /// **'Version {version} available'**
  String updateAvailable(String version);

  /// No description provided for @updateDownload.
  ///
  /// In en, this message translates to:
  /// **'Download'**
  String get updateDownload;

  /// No description provided for @updateAvailableStore.
  ///
  /// In en, this message translates to:
  /// **'Version {version} available — check Microsoft Store for the new version'**
  String updateAvailableStore(String version);

  /// No description provided for @updateTooltip.
  ///
  /// In en, this message translates to:
  /// **'New version available — check for updates in Settings'**
  String get updateTooltip;

  /// No description provided for @sectionSupport.
  ///
  /// In en, this message translates to:
  /// **'SUPPORT'**
  String get sectionSupport;

  /// No description provided for @donateLabel.
  ///
  /// In en, this message translates to:
  /// **'Buy me a coffee'**
  String get donateLabel;

  /// No description provided for @donateDescription.
  ///
  /// In en, this message translates to:
  /// **'LinkUnbound is free and always will be. If it saves you time, consider supporting development.'**
  String get donateDescription;

  /// No description provided for @sectionOtherTools.
  ///
  /// In en, this message translates to:
  /// **'OTHER TOOLS'**
  String get sectionOtherTools;

  /// No description provided for @otherToolCopyPaste.
  ///
  /// In en, this message translates to:
  /// **'CopyPaste'**
  String get otherToolCopyPaste;

  /// No description provided for @otherToolCopyPasteDescription.
  ///
  /// In en, this message translates to:
  /// **'Free, open source clipboard manager for Windows, macOS and Linux. Same philosophy: no ads, no telemetry, everything local.'**
  String get otherToolCopyPasteDescription;

  /// No description provided for @edgeWarningTitle.
  ///
  /// In en, this message translates to:
  /// **'Microsoft Edge detected'**
  String get edgeWarningTitle;

  /// No description provided for @edgeWarningBody.
  ///
  /// In en, this message translates to:
  /// **'Microsoft Teams, Outlook, and other Microsoft 365 apps may open links directly in Edge, ignoring your default browser. This is a Microsoft design decision that LinkUnbound cannot override.'**
  String get edgeWarningBody;

  /// No description provided for @edgeWarningNote.
  ///
  /// In en, this message translates to:
  /// **'You can change this behavior from each app\'s settings. Some organizations enforce Edge through group policies.'**
  String get edgeWarningNote;

  /// No description provided for @edgeWarningDismiss.
  ///
  /// In en, this message translates to:
  /// **'Got it, don\'t show again'**
  String get edgeWarningDismiss;

  /// No description provided for @sectionMaintenance.
  ///
  /// In en, this message translates to:
  /// **'MAINTENANCE'**
  String get sectionMaintenance;

  /// No description provided for @exportDiagnosticsLabel.
  ///
  /// In en, this message translates to:
  /// **'Export diagnostics'**
  String get exportDiagnosticsLabel;

  /// No description provided for @exportDiagnosticsDescription.
  ///
  /// In en, this message translates to:
  /// **'Generate a ZIP with system info, registry data, and logs for troubleshooting'**
  String get exportDiagnosticsDescription;

  /// No description provided for @errorStartupToggle.
  ///
  /// In en, this message translates to:
  /// **'Could not change startup setting'**
  String get errorStartupToggle;

  /// No description provided for @errorUnregister.
  ///
  /// In en, this message translates to:
  /// **'Could not unregister LinkUnbound'**
  String get errorUnregister;

  /// No description provided for @errorExportDiagnostics.
  ///
  /// In en, this message translates to:
  /// **'Could not export diagnostics'**
  String get errorExportDiagnostics;

  /// No description provided for @errorResetConfig.
  ///
  /// In en, this message translates to:
  /// **'Could not reset configuration'**
  String get errorResetConfig;

  /// No description provided for @sectionAccessibility.
  ///
  /// In en, this message translates to:
  /// **'ACCESSIBILITY'**
  String get sectionAccessibility;

  /// No description provided for @globalHotkeyLabel.
  ///
  /// In en, this message translates to:
  /// **'Global shortcut to open settings'**
  String get globalHotkeyLabel;

  /// No description provided for @globalHotkeyNone.
  ///
  /// In en, this message translates to:
  /// **'None (disabled)'**
  String get globalHotkeyNone;

  /// No description provided for @hideTrayLabel.
  ///
  /// In en, this message translates to:
  /// **'Hide tray / menu bar icon'**
  String get hideTrayLabel;

  /// No description provided for @hideTraySubtitleNoHotkey.
  ///
  /// In en, this message translates to:
  /// **'Requires a global shortcut to be configured first'**
  String get hideTraySubtitleNoHotkey;

  /// No description provided for @hideTraySubtitleMac.
  ///
  /// In en, this message translates to:
  /// **'You can also reach the app by relaunching it from Applications'**
  String get hideTraySubtitleMac;

  /// No description provided for @hideTraySubtitleWindows.
  ///
  /// In en, this message translates to:
  /// **'You can reach the app via its global shortcut'**
  String get hideTraySubtitleWindows;

  /// Shown in the picker when no browsers are configured
  ///
  /// In en, this message translates to:
  /// **'No browsers detected. Open Settings to add one.'**
  String get pickerNoBrowsers;

  /// Settings section header for Microsoft internal link capture
  ///
  /// In en, this message translates to:
  /// **'INTERNAL LINKS'**
  String get sectionInternalLinks;

  /// Toggle label for microsoft-edge: protocol capture (Windows only)
  ///
  /// In en, this message translates to:
  /// **'Capture links from Microsoft apps'**
  String get edgeProtocolLabel;

  /// Explains what capturing the microsoft-edge: protocol does
  ///
  /// In en, this message translates to:
  /// **'Teams, Outlook and Start menu search open links through Edge directly, bypassing the default browser. Enable this to let LinkUnbound offer a choice for those links too.'**
  String get edgeProtocolDescription;

  /// Hint in the picker footer: holding Shift opens a private window
  ///
  /// In en, this message translates to:
  /// **'Shift = private'**
  String get pickerPrivateHint;

  /// Checkbox label when the originating app is known
  ///
  /// In en, this message translates to:
  /// **'Always open links from {app} here'**
  String alwaysOpenFromApp(String app);

  /// Handler self-diagnostics in Settings
  ///
  /// In en, this message translates to:
  /// **'Link capture problem detected'**
  String get diagnosticsTitle;

  /// Handler self-diagnostics in Settings
  ///
  /// In en, this message translates to:
  /// **'The registered handler points somewhere else, so links will not reach LinkUnbound.'**
  String get diagnosticsStaleHandler;

  /// Handler self-diagnostics in Settings
  ///
  /// In en, this message translates to:
  /// **'Running from a local build, which never registers itself. Install the app to capture links.'**
  String get diagnosticsDevBuild;

  /// Handler self-diagnostics in Settings
  ///
  /// In en, this message translates to:
  /// **'Repair'**
  String get diagnosticsRepair;

  /// Handler self-diagnostics in Settings
  ///
  /// In en, this message translates to:
  /// **'Registration repaired.'**
  String get diagnosticsRepaired;

  /// Handler self-diagnostics in Settings
  ///
  /// In en, this message translates to:
  /// **'Could not repair the registration.'**
  String get diagnosticsRepairFailed;

  /// Rule label when scoped to the originating app
  ///
  /// In en, this message translates to:
  /// **'Links from {app}'**
  String ruleFromApp(String app);
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en', 'es'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
    case 'es':
      return AppLocalizationsEs();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
