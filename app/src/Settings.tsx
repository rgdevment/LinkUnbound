import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { type Key, type Language, Speaking, spoken, useWords } from "./i18n";
import About from "./settings/About";
import Application from "./settings/Application";
import Browsers from "./settings/Browsers";
import Maintenance from "./settings/Maintenance";
import { Card, Line, Section, Switch } from "./settings/parts";
import Rules from "./settings/Rules";

type Association = { scheme: string; held: boolean };
type Health = "fine" | "stale" | "build_tree" | "wrong_binary" | "not_registered";
type SystemState = {
  registered: boolean;
  is_default: boolean;
  associations: Association[];
  starts_with_system: boolean;
  startup_is_ours: boolean;
  health: Health;
  registered_path: string | null;
  edge_installed: boolean;
};

type Spoken = { language: string };

const AILMENT: Partial<Record<Health, { what: Key; fix: Key | null }>> = {
  stale: { what: "healthStale", fix: "healthRepair" },
  build_tree: { what: "healthBuildTree", fix: null },
  wrong_binary: { what: "healthWrongBinary", fix: "healthRepair" },
};

type Page = "links" | "rules" | "browsers" | "app" | "care" | "about";

const PAGES: { id: Page; label: Key; icon: React.ReactNode }[] = [
  {
    id: "links",
    label: "navLinks",
    icon: (
      <>
        <path d="M10 13a5 5 0 0 0 7.5.5l3-3a5 5 0 0 0-7-7l-1.7 1.7" />
        <path d="M14 11a5 5 0 0 0-7.5-.5l-3 3a5 5 0 0 0 7 7l1.7-1.7" />
      </>
    ),
  },
  { id: "rules", label: "navRules", icon: <path d="M4 6h16M4 12h11M4 18h7" /> },
  {
    id: "browsers",
    label: "navBrowsers",
    icon: (
      <>
        <circle cx="12" cy="12" r="9" />
        <path d="M3 12h18M12 3a15 15 0 0 1 0 18M12 3a15 15 0 0 0 0 18" />
      </>
    ),
  },
  {
    id: "app",
    label: "navApp",
    icon: (
      <>
        <circle cx="12" cy="12" r="3" />
        <path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9l2.2 2.2M16.9 16.9l2.2 2.2M19.1 4.9l-2.2 2.2M7.1 16.9l-2.2 2.2" />
      </>
    ),
  },
  {
    id: "care",
    label: "navCare",
    icon: (
      <>
        <path d="M14.7 6.3a4 4 0 0 0 5 5l-9 9a2.8 2.8 0 0 1-4-4z" />
        <path d="M17.5 3.5 21 7" />
      </>
    ),
  },
  {
    id: "about",
    label: "navAbout",
    icon: (
      <>
        <circle cx="12" cy="12" r="9" />
        <path d="M12 16v-4M12 8h.01" />
      </>
    ),
  },
];

function Links({
  state,
  change,
  run,
}: {
  state: SystemState | null;
  change: (command: string, enabled: boolean) => void;
  run: (command: string) => void;
}) {
  const t = useWords();
  const held = state?.associations.filter((a) => a.held).length ?? 0;
  const total = state?.associations.length ?? 0;
  const ok = state?.is_default ?? false;
  const ailment = state ? AILMENT[state.health] : undefined;

  return (
    <>
      {ailment && (
        <div className="rounded-lg border border-[#A85B14]/30 bg-[#A85B14]/[0.07] p-3.5 dark:border-[#E9A05C]/30 dark:bg-[#E9A05C]/[0.07]">
          <p className="text-[12.5px] font-semibold text-[#A85B14] dark:text-[#E9A05C]">
            {t("healthTitle")}
          </p>
          <p className="mt-1 text-[11.5px] text-neutral-600 dark:text-[#98A0B4]">
            {t(ailment.what)}
          </p>
          {state?.registered_path && (
            <p className="mt-1 font-mono text-[10.5px] break-all text-neutral-500 dark:text-[#8B92A1]">
              {t("healthPointsAt", state.registered_path)}
            </p>
          )}
          {ailment.fix && (
            <button
              type="button"
              onClick={() => run("system_repair")}
              className="mt-2.5 rounded-md bg-[#A85B14] px-3 py-1.5 text-[11.5px] font-medium text-white dark:bg-[#E9A05C] dark:text-[#12141B]"
            >
              {t(ailment.fix)}
            </button>
          )}
        </div>
      )}
      <Section title={t("sectionDefault")}>
        <div className="flex items-center gap-2 rounded-lg border border-black/[0.08] bg-black/[0.02] px-3 py-2.5 text-[11.5px] dark:border-white/[0.08] dark:bg-white/[0.02]">
          <span
            className={`h-[7px] w-[7px] shrink-0 rounded-full ${ok ? "bg-[#1E7A52] dark:bg-[#4CC38A]" : "bg-[#A85B14] dark:bg-[#E9A05C]"}`}
          />
          <span className="flex-1">{ok ? t("defaultYes") : t("defaultNo")}</span>
          <span className="text-neutral-500 dark:text-[#8B92A1]">
            {t("associations", String(held), String(total))}
          </span>
        </div>
        <Card>
          <Line title={t("offerTitle")} note={t("offerNote")}>
            <Switch
              on={state?.registered ?? false}
              label={t("offerTitle")}
              onChange={(next) => change("system_set_registered", next)}
            />
          </Line>
        </Card>
        {!ok && (
          <Card>
            <Line title={t("chooseTitle")} note={t("chooseNote")}>
              <button
                type="button"
                onClick={() => run("system_open_default_apps")}
                className="shrink-0 rounded-md border border-black/[0.12] px-3 py-1.5 text-[11.5px] dark:border-white/[0.12]"
              >
                {t("chooseGo")}
              </button>
            </Line>
          </Card>
        )}
      </Section>

      {state?.edge_installed && (
        <Section title={t("edgeTitle")}>
          <div className="rounded-lg border border-black/[0.08] bg-black/[0.015] p-3.5 text-[11.5px] leading-relaxed text-neutral-600 dark:border-white/[0.08] dark:bg-white/[0.02] dark:text-[#98A0B4]">
            <p>{t("edgeBody")}</p>
            <p className="mt-2">{t("edgeRelief")}</p>
          </div>
        </Section>
      )}
    </>
  );
}

export default function Settings() {
  const [language, setLanguage] = useState<Language>("es");

  useEffect(() => {
    void invoke<Spoken>("prefs_get")
      .then(({ language }) => setLanguage(spoken(language)))
      .catch(noop);
  }, []);

  // A screen reader picks its voice from this, so leaving it at the document's
  // default reads Spanish with an English engine.
  useEffect(() => {
    document.documentElement.lang = language;
  }, [language]);

  return (
    <Speaking value={language}>
      <Shell onLanguage={setLanguage} />
    </Speaking>
  );
}

function Shell({ onLanguage }: { onLanguage: (next: Language) => void }) {
  const t = useWords();
  const [page, setPage] = useState<Page>("links");
  const [state, setState] = useState<SystemState | null>(null);
  const [problem, setProblem] = useState<string | null>(null);

  useEffect(() => {
    void invoke<SystemState>("system_state").then(setState).catch(noop);
  }, []);

  const change = useCallback((command: string, enabled: boolean) => {
    setProblem(null);
    void invoke<SystemState>(command, { enabled })
      .then(setState)
      .catch((e: unknown) => setProblem(String(e)));
  }, []);

  const run = useCallback((command: string) => {
    setProblem(null);
    void invoke<SystemState | null>(command)
      .then((next) => {
        if (next) setState(next);
      })
      .catch((e: unknown) => setProblem(String(e)));
  }, []);

  return (
    <div className="flex h-full bg-white text-[#16181D] dark:bg-[#191A1F] dark:text-[#F0F1F4]">
      <nav
        aria-label={t("navLabel")}
        className="flex w-44 shrink-0 flex-col gap-0.5 border-r border-black/[0.08] bg-black/[0.02] p-2 dark:border-white/[0.08] dark:bg-white/[0.02]"
      >
        {PAGES.map((p) => (
          <button
            key={p.id}
            type="button"
            aria-current={p.id === page ? "page" : undefined}
            onClick={() => setPage(p.id)}
            className={`flex items-center gap-2.5 rounded-md px-2.5 py-[7px] text-left text-[12.5px] transition ${
              p.id === page
                ? "bg-[#2F62D8]/[0.11] font-semibold text-[#2F62D8] dark:bg-[#6E9BFF]/[0.16] dark:text-[#6E9BFF]"
                : "text-neutral-500 hover:bg-black/[0.04] hover:text-inherit dark:text-[#8B92A1] dark:hover:bg-white/[0.06]"
            }`}
          >
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.8"
              strokeLinecap="round"
              className="h-[15px] w-[15px] shrink-0"
              aria-hidden="true"
            >
              {p.icon}
            </svg>
            {t(p.label)}
          </button>
        ))}
        <span className="mt-auto px-2.5 py-2 text-[10.5px] text-neutral-400 dark:text-[#646B7C]">
          {t("version", "2.0.0")}
        </span>
      </nav>

      <main className="flex min-w-0 flex-1 flex-col gap-4 overflow-y-auto p-5">
        <h1 className="text-[15px] font-semibold">
          {t(PAGES.find((p) => p.id === page)?.label ?? "navLinks")}
        </h1>
        {problem && (
          <p className="rounded-md bg-[#C0362F]/10 px-3 py-2 text-[11.5px] text-[#C0362F] dark:bg-[#FF8A85]/10 dark:text-[#FF8A85]">
            {problem}
          </p>
        )}
        {page === "links" && <Links state={state} change={change} run={run} />}
        {page === "rules" && <Rules />}
        {page === "browsers" && <Browsers />}
        {page === "app" && (
          <Application
            startsWithSystem={state?.starts_with_system ?? false}
            startupIsOurs={state?.startup_is_ours ?? true}
            onSystem={change}
            onLanguage={onLanguage}
          />
        )}
        {page === "care" && <Maintenance />}
        {page === "about" && <About />}
      </main>
    </div>
  );
}

function noop() {}
