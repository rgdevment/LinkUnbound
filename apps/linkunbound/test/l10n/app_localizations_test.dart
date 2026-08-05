import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:linkunbound/l10n/app_localizations.dart';
import 'package:linkunbound/l10n/app_localizations_en.dart';
import 'package:linkunbound/l10n/app_localizations_es.dart';

void _expectAllMessages(AppLocalizations l10n) {
  final values = <String>[
    l10n.exit,
    l10n.traySettings,
    l10n.copyUrl,
    l10n.alwaysOpenHere,
    l10n.tabGeneral,
    l10n.tabRules,
    l10n.tabAbout,
    l10n.tabMaintenance,
    l10n.sectionDefaultBrowser,
    l10n.isDefaultBrowser,
    l10n.notDefaultBrowser,
    l10n.setDefault,
    l10n.sectionStartup,
    l10n.launchAtStartup,
    l10n.sectionLanguage,
    l10n.languageAuto,
    l10n.languageEnglish,
    l10n.languageSpanish,
    l10n.sectionAppearance,
    l10n.themeLabel,
    l10n.themeSystem,
    l10n.themeLight,
    l10n.themeDark,
    l10n.sectionBrowsers,
    l10n.addBrowserTooltip,
    l10n.refreshBrowsersTooltip,
    l10n.menuEdit,
    l10n.menuDuplicate,
    l10n.menuRemove,
    l10n.refreshNoChanges,
    l10n.editBrowserTitle,
    l10n.addBrowserTitle,
    l10n.fieldName,
    l10n.fieldExecutablePath,
    l10n.fieldExtraArgs,
    l10n.fieldIconPath,
    l10n.fieldIconHint,
    l10n.cancel,
    l10n.add,
    l10n.save,
    l10n.confirm,
    l10n.sectionUrlRules,
    l10n.noRulesYet,
    l10n.columnDomain,
    l10n.columnBrowser,
    l10n.deleteRuleTitle,
    l10n.delete,
    l10n.deleteRuleTooltip,
    l10n.sectionAbout,
    l10n.appDescription,
    l10n.licenseName,
    l10n.resetConfigLabel,
    l10n.resetConfigDescription,
    l10n.unregisterLabel,
    l10n.unregisterDescription,
    l10n.resetConfigTitle,
    l10n.resetConfigContent,
    l10n.reset,
    l10n.unregisterTitle,
    l10n.unregisterContent,
    l10n.unregisterAction,
    l10n.updateDownload,
    l10n.updateTooltip,
    l10n.sectionSupport,
    l10n.donateLabel,
    l10n.donateDescription,
    l10n.sectionOtherTools,
    l10n.otherToolCopyPaste,
    l10n.otherToolCopyPasteDescription,
    l10n.edgeWarningTitle,
    l10n.edgeWarningBody,
    l10n.edgeWarningNote,
    l10n.edgeWarningDismiss,
    l10n.sectionMaintenance,
    l10n.exportDiagnosticsLabel,
    l10n.exportDiagnosticsDescription,
  ];

  expect(values, everyElement(isNotEmpty));
  expect(l10n.refreshResult(2, 1), contains('2'));
  expect(l10n.refreshResult(2, 1), contains('1'));
  expect(l10n.deleteRuleContent('example.com'), contains('example.com'));
  expect(l10n.appVersion('1.2.3'), contains('1.2.3'));
  expect(l10n.updateAvailable('2.0.0'), contains('2.0.0'));
}

void main() {
  group('AppLocalizations delegate', () {
    test('supports English and Spanish only', () {
      expect(AppLocalizations.delegate.isSupported(const Locale('en')), isTrue);
      expect(AppLocalizations.delegate.isSupported(const Locale('es')), isTrue);
      expect(
        AppLocalizations.delegate.isSupported(const Locale('fr')),
        isFalse,
      );
    });

    test('loads Spanish strings', () async {
      final l10n = await AppLocalizations.delegate.load(const Locale('es'));
      expect(l10n.exit, 'Salir');
      expect(l10n.traySettings, 'Configuración');
    });

    test('lookup throws for unsupported locale', () {
      expect(
        () => lookupAppLocalizations(const Locale('fr')),
        throwsA(isA<FlutterError>()),
      );
    });

    testWidgets('of returns localization from widget tree', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          locale: const Locale('es'),
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: Builder(builder: (_) => const _LocalizedText()),
        ),
      );

      expect(find.text('Salir'), findsOneWidget);
    });
  });

  group('AppLocalizations messages', () {
    test('English getters and formatters are populated', () {
      final l10n = AppLocalizationsEn();
      _expectAllMessages(l10n);
      expect(l10n.exit, 'Exit');
      expect(l10n.launchAtStartup, 'Launch at system startup');
      expect(l10n.refreshResult(3, 2), '3 added, 2 removed');
    });

    test('Spanish getters and formatters are populated', () {
      final l10n = AppLocalizationsEs();
      _expectAllMessages(l10n);
      expect(l10n.exit, 'Salir');
      expect(l10n.launchAtStartup, 'Iniciar con el sistema');
      expect(l10n.refreshResult(3, 2), '3 añadidos, 2 eliminados');
    });
  });
}

class _LocalizedText extends StatelessWidget {
  const _LocalizedText();

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return Text(l10n.exit, textDirection: TextDirection.ltr);
  }
}
