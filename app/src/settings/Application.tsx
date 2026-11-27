import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { type Key, type Language, spoken, useSpoken, useWords } from "../i18n";
import { saidPlainly } from "../refusal";
import { Card, Choice, Line, Section, Switch } from "./parts";

type Locale = "system" | "spanish" | "english";

type Preferences = {
  schema_version: number;
  locale: Locale;
  shortcut: string | null;
  notify_on_rule: boolean;
  looks_for_updates: boolean;
  updates_automatically?: boolean | null;
};

type Settings = {
  prefs: Preferences;
  shortcut_held: string | null;
  language: string;
  installs_here: boolean;
};

const LOCALES: { id: Locale; label: Key }[] = [
  { id: "system", label: "localeAuto" },
  { id: "spanish", label: "localeSpanish" },
  { id: "english", label: "localeEnglish" },
];

const MODIFIERS = new Set(["Control", "Alt", "Shift", "Meta"]);

const RESIDENT_ANSWERS_WITHIN = 1400;

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

export default function Application({
  startsWithSystem,
  startupIsOurs,
  onSystem,
  onLanguage,
}: {
  startsWithSystem: boolean;
  startupIsOurs: boolean;
  onSystem: (command: string, enabled: boolean) => void;
  onLanguage: (next: Language) => void;
}) {
  const t = useWords();
  const language = useSpoken();
  const [settings, setSettings] = useState<Settings | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [capturing, setCapturing] = useState(false);

  const apply = (patch: Partial<Preferences>) => {
    if (!settings) return;
    void invoke<Settings>("prefs_set", { prefs: { ...settings.prefs, ...patch } })
      .then((next) => {
        setSettings(next);
        onLanguage(spoken(next.language));
        setProblem(null);
        if ("shortcut" in patch) askAgainOnceTheResidentHasIt();
      })
      .catch((e: unknown) => setProblem(saidPlainly(language, e)))
      .finally(() => setCapturing(false));
  };

  const setAutomatically = (on: boolean) => {
    void invoke<Settings>("updates_automatically_set", { on })
      .then((next) => {
        setSettings(next);
        setProblem(null);
      })
      .catch((e: unknown) => setProblem(saidPlainly(language, e)));
  };

  const askAgainOnceTheResidentHasIt = () => {
    setTimeout(() => {
      void invoke<Settings>("prefs_get").then(setSettings).catch(noop);
    }, RESIDENT_ANSWERS_WITHIN);
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
        <p className="rounded-md bg-danger/10 px-3 py-2 text-sm text-danger">{problem}</p>
      )}

      <Section title={t("appLanguage")}>
        <Card>
          <Line title={t("appLanguageLine")}>
            <Choice
              options={LOCALES}
              value={prefs.locale}
              label={t("appLanguageLine")}
              onPick={(locale) => apply({ locale })}
            />
          </Line>
        </Card>
      </Section>

      <Section title={t("appStartup")}>
        <Card>
          <Line
            title={t("startupTitle")}
            note={startupIsOurs ? t("startupOurs") : t("startupWindows")}
          >
            <Switch
              on={startsWithSystem}
              disabled={!startupIsOurs}
              label={t("startupTitle")}
              onChange={(next) => onSystem("system_set_startup", next)}
            />
          </Line>
        </Card>
      </Section>

      <Section title={t("appQuick")}>
        <Card>
          <Line
            title={t("shortcutTitle")}
            note={taken ? t("shortcutTaken") : prefs.shortcut ? t("shortcutHow") : t("shortcutOff")}
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
                className={`min-w-[124px] rounded-md border px-3 py-1.5 text-sm ${
                  capturing
                    ? "border-accent text-accent"
                    : taken
                      ? "border-danger/40 text-danger"
                      : "border-edge"
                }`}
              >
                {capturing ? t("shortcutPress") : (prefs.shortcut ?? t("shortcutNone"))}
              </button>
              {prefs.shortcut && (
                <button
                  type="button"
                  onClick={() => apply({ shortcut: null })}
                  className="text-sm text-muted"
                >
                  {t("remove")}
                </button>
              )}
            </div>
          </Line>
        </Card>
      </Section>

      <Section title={t("appUpdates")}>
        <Card>
          <Line title={t("looksTitle")} note={t("looksNote")}>
            <Switch
              on={prefs.looks_for_updates}
              label={t("looksTitle")}
              onChange={(looks_for_updates) => apply({ looks_for_updates })}
            />
          </Line>
          {settings.installs_here && (
            <Line
              title={t("automaticTitle")}
              note={prefs.looks_for_updates ? t("automaticNote") : t("automaticNeedsLooking")}
            >
              <Switch
                on={prefs.updates_automatically === true}
                disabled={!prefs.looks_for_updates}
                label={t("automaticTitle")}
                onChange={setAutomatically}
              />
            </Line>
          )}
        </Card>
      </Section>

      <Section title={t("appRule")}>
        <Card>
          <Line title={t("notifyTitle")} note={t("notifyNote")}>
            <Switch
              on={prefs.notify_on_rule}
              label={t("notifyTitle")}
              onChange={(notify_on_rule) => apply({ notify_on_rule })}
            />
          </Line>
        </Card>
      </Section>
    </>
  );
}

function noop() {}
