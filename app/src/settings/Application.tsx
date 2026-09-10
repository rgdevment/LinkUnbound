import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Card, Line, Section, Switch } from "./parts";

type Theme = "system" | "light" | "dark";
type Locale = "system" | "spanish" | "english";

type Preferences = {
  schema_version: number;
  theme: Theme;
  locale: Locale;
  shortcut: string | null;
  hide_tray: boolean;
  notify_on_rule: boolean;
};

type Settings = { prefs: Preferences; shortcut_held: string | null };

const THEMES: { id: Theme; label: string }[] = [
  { id: "system", label: "Automático" },
  { id: "light", label: "Claro" },
  { id: "dark", label: "Oscuro" },
];

const LOCALES: { id: Locale; label: string }[] = [
  { id: "system", label: "Automático" },
  { id: "spanish", label: "Español" },
  { id: "english", label: "Inglés" },
];

const MODIFIERS = new Set(["Control", "Alt", "Shift", "Meta"]);

/// Tauri parses the combination, and it wants `Control`, `Super` and the
/// physical key name rather than whatever the layout produced.
function combinationOf(e: React.KeyboardEvent): string | null {
  if (MODIFIERS.has(e.key)) return null;
  const parts: string[] = [];
  if (e.ctrlKey) parts.push("Control");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (e.metaKey) parts.push("Super");

  const named = /^(F\d{1,2}|Space|Escape|Enter|Tab)$/.exec(e.code);
  const key = named ? e.code : /^(?:Key|Digit)(.)$/.exec(e.code)?.[1];
  if (!key) return null;
  if (parts.length === 0 && !named?.[0]?.startsWith("F")) return null;
  parts.push(key);
  return parts.join("+");
}

function Choice<T extends string>({
  options,
  value,
  label,
  onPick,
}: {
  options: { id: T; label: string }[];
  value: T;
  label: string;
  onPick: (next: T) => void;
}) {
  return (
    <fieldset className="flex shrink-0 gap-1">
      <legend className="sr-only">{label}</legend>
      {options.map((o) => (
        <label key={o.id} className="contents">
          <input
            type="radio"
            name={label}
            className="peer sr-only"
            checked={o.id === value}
            onChange={() => onPick(o.id)}
          />
          <span
            className={`cursor-pointer rounded-md px-2.5 py-1 text-[11.5px] transition peer-focus-visible:outline-2 peer-focus-visible:outline-offset-1 peer-focus-visible:outline-[#2F62D8] dark:peer-focus-visible:outline-[#6E9BFF] ${
              o.id === value
                ? "bg-[#2F62D8]/[0.13] font-semibold text-[#2F62D8] dark:bg-[#6E9BFF]/[0.17] dark:text-[#6E9BFF]"
                : "bg-black/[0.045] text-neutral-500 dark:bg-white/[0.06] dark:text-[#8B92A1]"
            }`}
          >
            {o.label}
          </span>
        </label>
      ))}
    </fieldset>
  );
}

export default function Application({
  startsWithSystem,
  startupIsOurs,
  onSystem,
}: {
  startsWithSystem: boolean;
  startupIsOurs: boolean;
  onSystem: (command: string, enabled: boolean) => void;
}) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [capturing, setCapturing] = useState(false);

  const apply = (patch: Partial<Preferences>) => {
    if (!settings) return;
    void invoke<Settings>("prefs_set", { prefs: { ...settings.prefs, ...patch } })
      .then((next) => {
        setSettings(next);
        setProblem(null);
      })
      .catch((e: unknown) => setProblem(String(e)))
      .finally(() => setCapturing(false));
  };

  useEffect(() => {
    void invoke<Settings>("prefs_get").then(setSettings).catch(noop);
  }, []);

  if (!settings) return null;
  const { prefs, shortcut_held } = settings;
  const taken = prefs.shortcut !== null && shortcut_held === null;

  return (
    <>
      {problem && (
        <p className="rounded-md bg-[#C0362F]/10 px-3 py-2 text-[11.5px] text-[#C0362F] dark:bg-[#FF8A85]/10 dark:text-[#FF8A85]">
          {problem}
        </p>
      )}

      <Section title="Apariencia">
        <Card>
          <Line title="Tema">
            <Choice
              options={THEMES}
              value={prefs.theme}
              label="Tema"
              onPick={(theme) => apply({ theme })}
            />
          </Line>
          <Line title="Idioma" note="Requiere reiniciar la aplicación">
            <Choice
              options={LOCALES}
              value={prefs.locale}
              label="Idioma"
              onPick={(locale) => apply({ locale })}
            />
          </Line>
        </Card>
      </Section>

      <Section title="Inicio">
        <Card>
          <Line
            title="Iniciar con el sistema"
            note={
              startupIsOurs
                ? "El primer enlace de cada sesión se abre al instante"
                : "Gestionado desde Configuración de Windows > Aplicaciones de inicio"
            }
          >
            <Switch
              on={startsWithSystem}
              disabled={!startupIsOurs}
              label="Iniciar con el sistema"
              onChange={(next) => onSystem("system_set_startup", next)}
            />
          </Line>
        </Card>
      </Section>

      <Section title="Acceso rápido">
        <Card>
          <Line
            title="Atajo para abrir los ajustes"
            note={
              taken
                ? "Otra aplicación ya usa esa combinación"
                : prefs.shortcut
                  ? "Pulsa el botón y luego la combinación que quieras"
                  : "Desactivado"
            }
          >
            <div className="flex shrink-0 items-center gap-2">
              <button
                type="button"
                onKeyDown={(e) => {
                  if (!capturing) return;
                  e.preventDefault();
                  if (e.key === "Escape") {
                    setCapturing(false);
                    return;
                  }
                  const next = combinationOf(e);
                  if (next) apply({ shortcut: next });
                }}
                onClick={() => setCapturing(true)}
                onBlur={() => setCapturing(false)}
                className={`min-w-[124px] rounded-md border px-3 py-1.5 text-[11.5px] ${
                  capturing
                    ? "border-[#2F62D8] text-[#2F62D8] dark:border-[#6E9BFF] dark:text-[#6E9BFF]"
                    : taken
                      ? "border-[#C0362F]/40 text-[#C0362F] dark:border-[#FF8A85]/40 dark:text-[#FF8A85]"
                      : "border-black/[0.12] dark:border-white/[0.12]"
                }`}
              >
                {capturing ? "Pulsa una combinación…" : (prefs.shortcut ?? "Sin atajo")}
              </button>
              {prefs.shortcut && (
                <button
                  type="button"
                  onClick={() => apply({ shortcut: null })}
                  className="text-[11.5px] text-neutral-500 dark:text-[#8B92A1]"
                >
                  Quitar
                </button>
              )}
            </div>
          </Line>
          <Line
            title="Ocultar el icono de la bandeja"
            note={
              prefs.shortcut
                ? "Seguirás pudiendo abrir los ajustes con el atajo"
                : "Necesitas un atajo antes de poder ocultarlo"
            }
          >
            <Switch
              on={prefs.hide_tray}
              disabled={!prefs.shortcut}
              label="Ocultar el icono de la bandeja"
              onChange={(hide_tray) => apply({ hide_tray })}
            />
          </Line>
        </Card>
      </Section>

      <Section title="Cuando una regla decide">
        <Card>
          <Line
            title="Avisar cuando una regla abre sin preguntar"
            note="Un aviso breve, con la opción de deshacer"
          >
            <Switch
              on={prefs.notify_on_rule}
              label="Avisar cuando una regla abre sin preguntar"
              onChange={(notify_on_rule) => apply({ notify_on_rule })}
            />
          </Line>
        </Card>
      </Section>
    </>
  );
}

function noop() {}
