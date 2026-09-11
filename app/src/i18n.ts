import { createContext, useContext } from "react";

export type Language = "es" | "en";

const ES = {
  navLinks: "Enlaces",
  navRules: "Reglas",
  navBrowsers: "Navegadores",
  navApp: "Aplicación",
  navCare: "Mantenimiento",
  navAbout: "Acerca de",
  navLabel: "Secciones",
  version: "Versión {}",

  healthTitle: "LinkUnbound no está recibiendo los enlaces",
  healthStale:
    "El registro apunta a una copia de LinkUnbound que ya no está ahí. Los enlaces no llegarán hasta repararlo.",
  healthBuildTree:
    "Estás ejecutando una copia recién compilada. Windows no puede confiar los enlaces a una ruta que desaparece al limpiar el proyecto.",
  healthRepair: "Reparar",
  sectionDefault: "Navegador predeterminado",
  defaultYes: "LinkUnbound recibe los enlaces de este equipo",
  defaultNo: "Windows todavía no envía los enlaces aquí",
  associations: "{} de {} asociaciones",
  offerTitle: "Ofrecerse como navegador",
  offerNote: "Aparece en la lista de Windows para que puedas elegirlo",
  chooseTitle: "Elegir LinkUnbound en Windows",
  chooseNote: "Solo tú puedes fijar el predeterminado, y se hace en el panel del sistema",
  chooseGo: "Abrir Windows",
  edgeTitle: "Por qué Teams y Outlook abren Edge",
  edgeBody:
    "Algunas aplicaciones de Microsoft no abren los enlaces como lo haría cualquier otra: se los pasan directamente a Edge por un canal propio, saltándose el navegador predeterminado del sistema. Windows 11 ya no permite que otra aplicación se ponga en medio de ese canal, así que no hay nada que LinkUnbound pueda hacer para interceptarlo.",
  edgeRelief:
    "Lo que sí hace: cuando uno de esos enlaces llega envuelto en un enlace protegido, LinkUnbound lo desenvuelve y te ofrece el destino real en lugar del intermediario.",

  kindAny: "Cualquiera",
  kindUrl: "Esta URL",
  kindHost: "Subdominio",
  kindSite: "Sitio",
  kindFrom: "Desde",
  rulesEmptyTitle: "Todavía no hay ninguna regla",
  rulesEmptyBody:
    "Se crean desde el selector, eligiendo un alcance distinto de «Solo esta vez» antes de pulsar un navegador.",
  rulesOne: "1 regla",
  rulesMany: "{} reglas",
  rulesFooter:
    "Entre reglas que alcanzan lo mismo gana la primera. Las marcadas «Desde» vienen de la versión 1.4 y el selector ya no las crea.",
  rulePrivate: "privada",
  ruleUp: "Subir la regla de {}",
  ruleDown: "Bajar la regla de {}",
  ruleRemove: "Eliminar la regla de {}",

  browsersDetected: "Detectados en el equipo",
  browsersMine: "Añadidos por ti",
  browsersNone: "Windows no reporta ningún navegador instalado.",
  browsersAdd: "Añadir un navegador",
  profilesNone: "Sin perfiles",
  profilesOne: "1 perfil",
  profilesMany: "{} perfiles",
  privateYes: "admite ventana privada",
  privateNo: "sin ventana privada",
  hiddenFromPicker: "oculto del selector",
  fieldName: "Nombre",
  fieldExe: "Ruta del ejecutable",
  fieldArgs: "Argumentos adicionales (separados por espacios)",
  fieldArgsLabel: "Argumentos adicionales",
  fieldPrivate: "Argumento de ventana privada (--incognito, -private-window…)",
  fieldPrivateLabel: "Argumento de ventana privada",
  fieldPrivateHint: "Sin este argumento, el navegador no podrá abrir en privado desde el selector.",
  fieldIcon: "Ruta de icono personalizado (opcional)",
  fieldIconLabel: "Ruta de icono personalizado",
  browserUp: "Subir {}",
  browserDown: "Bajar {}",
  browserDuplicate: "Duplicar {}",
  browserEdit: "Editar {}",
  browserRemove: "Eliminar {}",
  browserShow: "Mostrar {} en el selector",

  appAppearance: "Apariencia",
  appTheme: "Tema",
  appLanguage: "Idioma",
  themeAuto: "Automático",
  themeLight: "Claro",
  themeDark: "Oscuro",
  localeAuto: "Automático",
  localeSpanish: "Español",
  localeEnglish: "Inglés",
  appStartup: "Inicio",
  startupTitle: "Iniciar con el sistema",
  startupOurs: "El primer enlace de cada sesión se abre al instante",
  startupWindows: "Gestionado desde Configuración de Windows > Aplicaciones de inicio",
  appQuick: "Acceso rápido",
  shortcutTitle: "Atajo para abrir los ajustes",
  shortcutTaken: "Otra aplicación ya usa esa combinación",
  shortcutHow: "Pulsa el botón y luego la combinación que quieras",
  shortcutOff: "Desactivado",
  shortcutPress: "Pulsa una combinación…",
  shortcutNone: "Sin atajo",
  hideTrayTitle: "Ocultar el icono de la bandeja",
  hideTrayWith: "Seguirás pudiendo abrir los ajustes con el atajo",
  hideTrayNeeds: "Necesitas un atajo antes de poder ocultarlo",
  appRule: "Cuando una regla decide",
  notifyTitle: "Avisar cuando una regla abre sin preguntar",
  notifyNote: "Un aviso breve, con la opción de deshacer",

  careReport: "Informar de un problema",
  reportTitle: "Guardar un informe de diagnóstico",
  reportNote: "Un archivo con el estado de la app y tus reglas, sin las direcciones que visitas",
  reportSaved: "Guardado en {}",
  careBrowsers: "Navegadores",
  rescanTitle: "Volver a buscar navegadores",
  rescanNote: "Útil si instalaste uno y no aparece en el selector",
  rescanGo: "Buscar",
  rescanBody:
    "Se olvida lo detectado y se vuelve a leer del registro. Los navegadores que añadiste a mano se conservan; lo que hayas ocultado se mostrará de nuevo.",
  careScratch: "Empezar de cero",
  resetTitle: "Restablecer la configuración",
  resetNote: "Borra todas las reglas y todos los navegadores",
  resetGo: "Restablecer",
  resetBody:
    "Se borran todas las reglas y todos los navegadores, incluidos los que añadiste. No se puede deshacer.",
  unregisterTitle: "Quitar LinkUnbound de Windows",
  unregisterNote: "Deja de ofrecerse como navegador en la lista del sistema",
  unregisterGo: "Quitar",
  unregisterBody:
    "Deja de ofrecerse como navegador. Es posible que después tengas que elegir otro predeterminado en la configuración de Windows.",
  doneWith: "Hecho: {}.",

  aboutTagline: "Elige tú en qué navegador se abre cada enlace.",
  aboutProject: "Proyecto",
  aboutSource: "Código fuente",
  aboutSourceNote: "GPL-3.0 · las aportaciones son bienvenidas",
  aboutIssue: "Informar de un problema",
  aboutIssueNote: "Errores, ideas y navegadores que no detecta",
  aboutSponsor: "Apoyar el desarrollo",
  aboutSponsorNote: "LinkUnbound es gratis y no lleva publicidad",
  open: "Abrir",

  save: "Guardar",
  cancel: "Cancelar",
  remove: "Quitar",
  errUnreachable: "Con la bandeja oculta y sin atajo no habría forma de volver aquí.",
} as const;

export type Key = keyof typeof ES;

const EN: Record<Key, string> = {
  navLinks: "Links",
  navRules: "Rules",
  navBrowsers: "Browsers",
  navApp: "Application",
  navCare: "Maintenance",
  navAbout: "About",
  navLabel: "Sections",
  version: "Version {}",

  healthTitle: "LinkUnbound is not receiving links",
  healthStale:
    "The registration points at a copy of LinkUnbound that is no longer there. Links will not arrive until it is repaired.",
  healthBuildTree:
    "You are running a freshly built copy. Windows cannot trust links to a path that disappears when the project is cleaned.",
  healthRepair: "Repair",
  sectionDefault: "Default browser",
  defaultYes: "LinkUnbound receives the links of this computer",
  defaultNo: "Windows is not sending links here yet",
  associations: "{} of {} associations",
  offerTitle: "Offer itself as a browser",
  offerNote: "Appears in the Windows list so that you can pick it",
  chooseTitle: "Choose LinkUnbound in Windows",
  chooseNote: "Only you can set the default, and it is done in the system panel",
  chooseGo: "Open Windows",
  edgeTitle: "Why Teams and Outlook open Edge",
  edgeBody:
    "Some Microsoft applications do not open links the way any other one would: they hand them straight to Edge down a channel of their own, skipping the system default browser. Windows 11 no longer lets another application stand in the middle of that channel, so there is nothing LinkUnbound can do to intercept it.",
  edgeRelief:
    "What it does do: when one of those links arrives wrapped in a protected link, LinkUnbound unwraps it and offers you the real destination instead of the middleman.",

  kindAny: "Anything",
  kindUrl: "This URL",
  kindHost: "Subdomain",
  kindSite: "Site",
  kindFrom: "From",
  rulesEmptyTitle: "No rules yet",
  rulesEmptyBody:
    "They are made from the picker, by choosing a reach other than «Just this time» before pressing a browser.",
  rulesOne: "1 rule",
  rulesMany: "{} rules",
  rulesFooter:
    "Between rules that reach the same thing, the first one wins. The ones marked «From» come from version 1.4 and the picker no longer makes them.",
  rulePrivate: "private",
  ruleUp: "Move the rule for {} up",
  ruleDown: "Move the rule for {} down",
  ruleRemove: "Delete the rule for {}",

  browsersDetected: "Detected on this computer",
  browsersMine: "Added by you",
  browsersNone: "Windows reports no installed browser.",
  browsersAdd: "Add a browser",
  profilesNone: "No profiles",
  profilesOne: "1 profile",
  profilesMany: "{} profiles",
  privateYes: "can open a private window",
  privateNo: "no private window",
  hiddenFromPicker: "hidden from the picker",
  fieldName: "Name",
  fieldExe: "Executable path",
  fieldArgs: "Extra arguments (space-separated)",
  fieldArgsLabel: "Extra arguments",
  fieldPrivate: "Private window argument (--incognito, -private-window…)",
  fieldPrivateLabel: "Private window argument",
  fieldPrivateHint:
    "Without this argument, the browser will not be able to open privately from the picker.",
  fieldIcon: "Custom icon path (optional)",
  fieldIconLabel: "Custom icon path",
  browserUp: "Move {} up",
  browserDown: "Move {} down",
  browserDuplicate: "Duplicate {}",
  browserEdit: "Edit {}",
  browserRemove: "Remove {}",
  browserShow: "Show {} in the picker",

  appAppearance: "Appearance",
  appTheme: "Theme",
  appLanguage: "Language",
  themeAuto: "Automatic",
  themeLight: "Light",
  themeDark: "Dark",
  localeAuto: "Automatic",
  localeSpanish: "Spanish",
  localeEnglish: "English",
  appStartup: "Startup",
  startupTitle: "Launch at system startup",
  startupOurs: "The first link of each session opens instantly",
  startupWindows: "Managed from Windows Settings > Startup apps",
  appQuick: "Quick access",
  shortcutTitle: "Shortcut to open settings",
  shortcutTaken: "Another application already uses that combination",
  shortcutHow: "Press the button and then whatever combination you want",
  shortcutOff: "Disabled",
  shortcutPress: "Press a combination…",
  shortcutNone: "No shortcut",
  hideTrayTitle: "Hide the tray icon",
  hideTrayWith: "You will still be able to open settings with the shortcut",
  hideTrayNeeds: "You need a shortcut before you can hide it",
  appRule: "When a rule decides",
  notifyTitle: "Warn when a rule opens without asking",
  notifyNote: "A brief notice, with the option to undo",

  careReport: "Report a problem",
  reportTitle: "Save a diagnostic report",
  reportNote: "A file with the state of the app and your rules, without the addresses you visit",
  reportSaved: "Saved to {}",
  careBrowsers: "Browsers",
  rescanTitle: "Look for browsers again",
  rescanNote: "Useful if you installed one and it does not show up in the picker",
  rescanGo: "Look",
  rescanBody:
    "What was detected is forgotten and read from the registry again. The browsers you added by hand are kept; whatever you hid will show up again.",
  careScratch: "Start from scratch",
  resetTitle: "Reset the configuration",
  resetNote: "Deletes every rule and every browser",
  resetGo: "Reset",
  resetBody:
    "Every rule and every browser is deleted, including the ones you added. This cannot be undone.",
  unregisterTitle: "Remove LinkUnbound from Windows",
  unregisterNote: "Stops offering itself as a browser in the system list",
  unregisterGo: "Remove",
  unregisterBody:
    "It stops offering itself as a browser. You may have to pick another default afterwards in the Windows settings.",
  doneWith: "Done: {}.",

  aboutTagline: "You choose which browser each link opens in.",
  aboutProject: "Project",
  aboutSource: "Source code",
  aboutSourceNote: "GPL-3.0 · contributions are welcome",
  aboutIssue: "Report a problem",
  aboutIssueNote: "Bugs, ideas and browsers it fails to detect",
  aboutSponsor: "Support the development",
  aboutSponsorNote: "LinkUnbound is free and carries no advertising",
  open: "Open",

  save: "Save",
  cancel: "Cancel",
  remove: "Remove",
  errUnreachable: "With the tray hidden and no shortcut there would be no way back here.",
};

export const SPEECH: Record<Language, Record<Key, string>> = { es: ES, en: EN };

export function fill(template: string, ...values: string[]): string {
  let at = 0;
  return template.replace(/\{\}/g, () => values[at++] ?? "");
}

/// Spanish: the catalogue is authored in it, so no provider reads as written.
const Spoken = createContext<Language>("es");

export const Speaking = Spoken.Provider;

export function useWords(): (key: Key, ...values: string[]) => string {
  const language = useContext(Spoken);
  return (key, ...values) => fill(SPEECH[language][key], ...values);
}

export function spoken(language: string | undefined): Language {
  return language === "en" ? "en" : "es";
}
