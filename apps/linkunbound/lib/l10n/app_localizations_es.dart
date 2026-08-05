// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Spanish Castilian (`es`).
class AppLocalizationsEs extends AppLocalizations {
  AppLocalizationsEs([String locale = 'es']) : super(locale);

  @override
  String get exit => 'Salir';

  @override
  String get traySettings => 'Configuración';

  @override
  String get copyUrl => 'Copiar URL';

  @override
  String get alwaysOpenHere => 'Abrir siempre aquí';

  @override
  String get tabGeneral => 'General';

  @override
  String get tabRules => 'Reglas';

  @override
  String get tabAbout => 'Acerca de';

  @override
  String get tabMaintenance => 'Mantenimiento';

  @override
  String get sectionDefaultBrowser => 'NAVEGADOR PREDETERMINADO';

  @override
  String get isDefaultBrowser =>
      'LinkUnbound está configurado como navegador predeterminado';

  @override
  String get notDefaultBrowser =>
      'LinkUnbound no está configurado como navegador predeterminado';

  @override
  String get setDefault => 'Establecer';

  @override
  String get sectionStartup => 'INICIO';

  @override
  String get launchAtStartup => 'Iniciar con el sistema';

  @override
  String get startupManagedByWindows =>
      'Gestionado desde Configuración de Windows > Aplicaciones de inicio';

  @override
  String get sectionLanguage => 'IDIOMA';

  @override
  String get languageAuto => 'Automático (sistema)';

  @override
  String get languageEnglish => 'Inglés';

  @override
  String get languageSpanish => 'Español';

  @override
  String get sectionAppearance => 'APARIENCIA';

  @override
  String get themeLabel => 'Tema';

  @override
  String get themeSystem => 'Automático (sistema)';

  @override
  String get themeLight => 'Claro';

  @override
  String get themeDark => 'Oscuro';

  @override
  String get sectionBrowsers => 'NAVEGADORES';

  @override
  String get addBrowserTooltip => 'Añadir navegador personalizado';

  @override
  String get refreshBrowsersTooltip => 'Actualizar navegadores';

  @override
  String get menuEdit => 'Editar';

  @override
  String get menuDuplicate => 'Duplicar';

  @override
  String get menuRemove => 'Eliminar';

  @override
  String refreshResult(int added, int removed) {
    return '$added añadidos, $removed eliminados';
  }

  @override
  String get refreshNoChanges => 'Sin cambios detectados';

  @override
  String get editBrowserTitle => 'Editar navegador';

  @override
  String get addBrowserTitle => 'Añadir navegador personalizado';

  @override
  String get fieldName => 'Nombre';

  @override
  String get fieldExecutablePath => 'Ruta del ejecutable';

  @override
  String get fieldExtraArgs =>
      'Argumentos adicionales (separados por espacios)';

  @override
  String get fieldIconPath => 'Ruta de icono personalizado (opcional)';

  @override
  String get fieldIconHint => 'Dejar vacío para detectar desde el ejecutable';

  @override
  String get cancel => 'Cancelar';

  @override
  String get add => 'Añadir';

  @override
  String get save => 'Guardar';

  @override
  String get confirm => 'Confirmar';

  @override
  String get sectionUrlRules => 'REGLAS DE URL';

  @override
  String get noRulesYet =>
      'Sin reglas aún. Las reglas se crean desde el selector de navegadores al marcar \"Abrir siempre aquí\".';

  @override
  String get columnDomain => 'Dominio';

  @override
  String get columnBrowser => 'Navegador';

  @override
  String get deleteRuleTitle => 'Eliminar regla';

  @override
  String deleteRuleContent(String domain) {
    return '¿Eliminar la regla para \"$domain\"?';
  }

  @override
  String get delete => 'Eliminar';

  @override
  String get deleteRuleTooltip => 'Eliminar regla';

  @override
  String get sectionAbout => 'ACERCA DE';

  @override
  String appVersion(String version) {
    return 'Versión $version';
  }

  @override
  String get appDescription =>
      'Selector de navegadores de código abierto para Windows y macOS.';

  @override
  String get licenseName => 'Licencia GPL-3.0';

  @override
  String get resetConfigLabel => 'Restablecer configuración';

  @override
  String get resetConfigDescription =>
      'Limpiar navegadores y reglas, luego volver a escanear';

  @override
  String get unregisterLabel => 'Desregistrar LinkUnbound';

  @override
  String get unregisterDescription =>
      'Eliminar de la lista de navegadores de Windows';

  @override
  String get resetConfigTitle => 'Restablecer configuración';

  @override
  String get resetConfigContent =>
      'Esto eliminará todos los navegadores, reglas e iconos, luego volverá a escanear los navegadores instalados. ¿Continuar?';

  @override
  String get reset => 'Restablecer';

  @override
  String get unregisterTitle => 'Desregistrar LinkUnbound';

  @override
  String get unregisterContent =>
      'Esto eliminará LinkUnbound de la lista de navegadores de Windows. Es posible que necesites cambiar tu navegador predeterminado en la Configuración de Windows después. ¿Continuar?';

  @override
  String get unregisterAction => 'Desregistrar';

  @override
  String updateAvailable(String version) {
    return 'Versión $version disponible';
  }

  @override
  String get updateDownload => 'Descargar';

  @override
  String updateAvailableStore(String version) {
    return 'Versión $version disponible — verifica en Microsoft Store la nueva versión';
  }

  @override
  String get updateTooltip =>
      'Nueva versión disponible — revisa las actualizaciones en Ajustes';

  @override
  String get sectionSupport => 'APÓYANOS';

  @override
  String get donateLabel => 'Invítame un café';

  @override
  String get donateDescription =>
      'LinkUnbound es gratis y siempre lo será. Si te ahorra tiempo, considera apoyar el desarrollo.';

  @override
  String get sectionOtherTools => 'OTRAS HERRAMIENTAS';

  @override
  String get otherToolCopyPaste => 'CopyPaste';

  @override
  String get otherToolCopyPasteDescription =>
      'Gestor de portapapeles gratuito y de código abierto para Windows, macOS y Linux. Misma filosofía: sin anuncios, sin telemetría, todo local.';

  @override
  String get edgeWarningTitle => 'Microsoft Edge detectado';

  @override
  String get edgeWarningBody =>
      'Microsoft Teams, Outlook y otras apps de Microsoft 365 pueden abrir links directamente en Edge, ignorando tu navegador predeterminado. Esto es una decisión de diseño de Microsoft que LinkUnbound no puede evitar.';

  @override
  String get edgeWarningNote =>
      'Puedes cambiar este comportamiento desde la configuración de cada app. Algunas organizaciones fuerzan Edge a través de políticas de grupo.';

  @override
  String get edgeWarningDismiss => 'Entendido, no mostrar de nuevo';

  @override
  String get sectionMaintenance => 'MANTENIMIENTO';

  @override
  String get exportDiagnosticsLabel => 'Exportar diagnóstico';

  @override
  String get exportDiagnosticsDescription =>
      'Genera un ZIP con info del sistema, datos del registro y logs para diagnóstico';

  @override
  String get errorStartupToggle =>
      'No se pudo cambiar la configuración de inicio';

  @override
  String get errorUnregister => 'No se pudo desregistrar LinkUnbound';

  @override
  String get errorExportDiagnostics => 'No se pudo exportar el diagnóstico';

  @override
  String get errorResetConfig => 'No se pudo restablecer la configuración';

  @override
  String get sectionAccessibility => 'ACCESIBILIDAD';

  @override
  String get globalHotkeyLabel => 'Atajo global para abrir la configuración';

  @override
  String get globalHotkeyNone => 'Ninguno (desactivado)';

  @override
  String get hideTrayLabel => 'Ocultar icono de bandeja / barra de menú';

  @override
  String get hideTraySubtitleNoHotkey =>
      'Requiere configurar primero un atajo global';

  @override
  String get hideTraySubtitleMac =>
      'También puedes abrir la app relanzándola desde Aplicaciones';

  @override
  String get hideTraySubtitleWindows =>
      'Puedes abrir la app mediante su atajo global';

  @override
  String get pickerNoBrowsers =>
      'No se detectaron navegadores. Abre Ajustes para añadir uno.';

  @override
  String get sectionInternalLinks => 'ENLACES INTERNOS';

  @override
  String get edgeProtocolLabel =>
      'Capturar enlaces de aplicaciones de Microsoft';

  @override
  String get edgeProtocolDescription =>
      'Teams, Outlook y la búsqueda del menú Inicio abren los enlaces directamente en Edge, ignorando el navegador predeterminado. Actívalo para que LinkUnbound también ofrezca elegir en esos enlaces.';

  @override
  String get pickerPrivateHint => 'Mayús = privada';

  @override
  String alwaysOpenFromApp(String app) {
    return 'Abrir siempre aquí los enlaces de $app';
  }

  @override
  String get diagnosticsTitle => 'Problema de captura de enlaces detectado';

  @override
  String get diagnosticsStaleHandler =>
      'El manejador registrado apunta a otra ubicación, así que los enlaces no llegarán a LinkUnbound.';

  @override
  String get diagnosticsDevBuild =>
      'Se está ejecutando una compilación local, que nunca se registra. Instala la aplicación para capturar enlaces.';

  @override
  String get diagnosticsRepair => 'Reparar';

  @override
  String get diagnosticsRepaired => 'Registro reparado.';

  @override
  String get diagnosticsRepairFailed => 'No se pudo reparar el registro.';

  @override
  String ruleFromApp(String app) {
    return 'Enlaces de $app';
  }
}
