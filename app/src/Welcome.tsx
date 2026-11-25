import { invoke } from "@tauri-apps/api/core";
import { type ReactNode, useCallback, useEffect, useRef, useState } from "react";
import bear from "./assets/linkunbound.svg";
import { type Key, type Language, Speaking, spoken, useWords } from "./i18n";
import { type SystemState, useChooseUs } from "./system";

type Step = "hello" | "links" | "try";
type Progress = "idle" | "waiting" | "done";
type Prefs = { language: string; prefs: { shortcut: string | null } };

const EVERY_SECOND = 1000;
const LONG_ENOUGH_TO_READ = 1200;

export default function Welcome() {
  const [language, setLanguage] = useState<Language>("es");
  const [shortcut, setShortcut] = useState<string | null>(null);

  useEffect(() => {
    void invoke<Prefs>("prefs_get")
      .then((said) => {
        setLanguage(spoken(said.language));
        setShortcut(said.prefs.shortcut);
      })
      .catch(noop);
  }, []);

  useEffect(() => {
    document.documentElement.lang = language;
  }, [language]);

  return (
    <Speaking value={language}>
      <Tour shortcut={shortcut} />
    </Speaking>
  );
}

function Tour({ shortcut }: { shortcut: string | null }) {
  const [steps, setSteps] = useState<Step[] | null>(null);
  const [at, setAt] = useState(0);

  // The step that hands the links over is left out for someone who already did.
  useEffect(() => {
    void invoke<boolean>("system_is_default")
      .then((handing) => setSteps(handing ? ["hello", "try"] : ["hello", "links", "try"]))
      .catch(() => setSteps(["hello", "links", "try"]));
  }, []);

  useEffect(() => {
    if (steps) void invoke("settings_painted").catch(noop);
  }, [steps]);

  const close = useCallback(() => void invoke("welcome_close").catch(noop), []);
  const next = useCallback(() => setAt((was) => was + 1), []);

  useEffect(() => {
    const held = (e: KeyboardEvent) => {
      if (e.key === "Escape") close();
    };
    document.addEventListener("keydown", held);
    return () => document.removeEventListener("keydown", held);
  }, [close]);

  if (!steps) return null;
  const step = steps[Math.min(at, steps.length - 1)];
  const dots = <Dots count={steps.length} at={at} />;

  return (
    <div className="flex h-full flex-col bg-panel text-ink">
      {step === "hello" && <Hello dots={dots} onSkip={close} onNext={next} />}
      {step === "links" && <Links dots={dots} onNext={next} />}
      {step === "try" && <Try dots={dots} shortcut={shortcut} onDone={close} />}
    </div>
  );
}

function Hello({
  dots,
  onSkip,
  onNext,
}: {
  dots: ReactNode;
  onSkip: () => void;
  onNext: () => void;
}) {
  const t = useWords();
  return (
    <>
      <div className="flex flex-1 flex-col items-center justify-center gap-3.5 px-12 text-center">
        <img src={bear} alt="" className="bear h-[80px]" />
        <h1 className="text-xl font-semibold">{t("welcomeHelloTitle")}</h1>
        <p className="max-w-[460px] text-md leading-relaxed text-ink-2">{t("welcomeHelloBody")}</p>
        <ul className="mt-1 flex flex-wrap justify-center gap-2">
          {(["welcomePillLocal", "welcomePillAccount", "welcomePillBrowsers"] as Key[]).map(
            (pill) => (
              <li
                key={pill}
                className="rounded-full bg-accent/10 px-3 py-1 text-sm text-accent dark:bg-accent/15"
              >
                {t(pill)}
              </li>
            ),
          )}
        </ul>
      </div>
      <Footer
        quiet={t("welcomeSkip")}
        onQuiet={onSkip}
        dots={dots}
        strong={t("welcomeStart")}
        onStrong={onNext}
      />
    </>
  );
}

function Links({ dots, onNext }: { dots: ReactNode; onNext: () => void }) {
  const t = useWords();
  const [progress, setProgress] = useState<Progress>("idle");
  const chosen = useCallback(
    (found: SystemState) => setProgress(found.is_default ? "done" : "waiting"),
    [],
  );
  const { choose, problem } = useChooseUs(chosen);

  // Windows answers in its own window, so the only way to hear it is to keep asking.
  useEffect(() => {
    if (progress !== "waiting") return;
    const asking = setInterval(() => {
      void invoke<boolean>("system_is_default")
        .then((handing) => {
          if (handing) setProgress("done");
        })
        .catch(noop);
    }, EVERY_SECOND);
    return () => clearInterval(asking);
  }, [progress]);

  useEffect(() => {
    if (progress !== "done") return;
    const onwards = setTimeout(onNext, LONG_ENOUGH_TO_READ);
    return () => clearTimeout(onwards);
  }, [progress, onNext]);

  return (
    <>
      <Page over={t("welcomeLinkOver")} title={t("welcomeLinkTitle")} body={t("welcomeLinkBody")}>
        <ol
          className={`mt-1 flex flex-col gap-2 text-md ${progress === "done" ? "text-muted" : ""}`}
        >
          {(["welcomeLinkStep1", "welcomeLinkStep2", "welcomeLinkStep3"] as Key[]).map(
            (line, n) => (
              <li key={line} className="flex items-baseline gap-2.5">
                <span className="font-semibold text-accent">{n + 1}</span>
                <span>{t(line)}</span>
              </li>
            ),
          )}
        </ol>
        {progress === "waiting" && (
          <Waiting said={t("welcomeLinkWaiting")} note={t("welcomeLinkHint")} />
        )}
        {progress === "done" && (
          <Done said={t("welcomeLinkDone")} note={t("welcomeLinkDoneNote")} />
        )}
        {problem && (
          <p role="alert" className="text-sm text-danger">
            {problem}
          </p>
        )}
      </Page>
      <Footer
        quiet={progress === "done" ? null : t("welcomeLater")}
        onQuiet={onNext}
        dots={dots}
        strong={
          progress === "done"
            ? t("welcomeNext")
            : progress === "waiting"
              ? t("welcomeLinkAgain")
              : t("welcomeLinkGo")
        }
        onStrong={progress === "done" ? onNext : () => choose(null)}
      />
    </>
  );
}

function Try({
  dots,
  shortcut,
  onDone,
}: {
  dots: ReactNode;
  shortcut: string | null;
  onDone: () => void;
}) {
  const t = useWords();
  const [progress, setProgress] = useState<Progress>("idle");
  const [handing, setHanding] = useState(true);

  useEffect(() => {
    void invoke<boolean>("system_is_default").then(setHanding).catch(noop);
  }, []);

  // The picker lives in another process; it leaves word of each link it opens.
  useEffect(() => {
    if (progress !== "waiting") return;
    const asking = setInterval(() => {
      void invoke<boolean>("welcome_tried")
        .then((yes) => {
          if (yes) setProgress("done");
        })
        .catch(noop);
    }, EVERY_SECOND);
    return () => clearInterval(asking);
  }, [progress]);

  // Asked at the click: a link that goes to the usual browser never reaches the picker to wait for.
  const go = async () => {
    const now = await invoke<boolean>("system_is_default").catch(() => false);
    setHanding(now);
    await invoke("welcome_try").catch(noop);
    if (now) setProgress((was) => (was === "done" ? was : "waiting"));
  };

  return (
    <>
      <Page over={t("welcomeTryOver")} title={t("welcomeTryTitle")} body={t("welcomeTryBody")}>
        <div className="flex justify-center py-2">
          <button
            type="button"
            onClick={() => void go()}
            className="flex items-center gap-2 rounded-lg border border-accent/25 bg-accent/[0.06] px-4 py-3 text-md text-accent hover:bg-accent/10 dark:border-accent/30 dark:bg-accent/10"
          >
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              className="h-4 w-4"
              aria-hidden="true"
            >
              <path d="M10 13a5 5 0 0 0 7.5.5l3-3a5 5 0 0 0-7-7l-1.7 1.7" />
              <path d="M14 11a5 5 0 0 0-7.5-.5l-3 3a5 5 0 0 0 7 7l1.7-1.7" />
            </svg>
            rgdevment.dev
          </button>
        </div>
        {!handing && progress !== "done" && (
          <p className="text-sm text-caution">{t("welcomeTryNotYet")}</p>
        )}
        {progress === "waiting" && <Waiting said={t("welcomeTryWaiting")} />}
        {progress === "done" && (
          <Done
            said={t("welcomeTryDone")}
            note={shortcut ? t("welcomeTryDoneNote", shortcut) : undefined}
          />
        )}
      </Page>
      <Footer
        quiet={progress === "done" ? null : t("welcomeSkip")}
        onQuiet={onDone}
        dots={dots}
        strong={t("welcomeFinish")}
        onStrong={onDone}
      />
    </>
  );
}

function Page({
  over,
  title,
  body,
  children,
}: {
  over: string;
  title: string;
  body: string;
  children: ReactNode;
}) {
  return (
    <div className="flex flex-1 flex-col gap-3 px-10 pt-7 pb-4">
      <span className="text-xs font-semibold tracking-[0.08em] text-muted uppercase">{over}</span>
      <h1 className="text-xl font-semibold">{title}</h1>
      <p className="max-w-[500px] text-md leading-relaxed text-ink-2">{body}</p>
      {children}
    </div>
  );
}

function Waiting({ said, note }: { said: string; note?: string }) {
  return (
    <div
      role="status"
      className="mt-1.5 flex items-start gap-3 rounded-[10px] border border-black/[0.08] bg-black/[0.02] px-4 py-3.5 dark:border-white/[0.08] dark:bg-white/[0.03]"
    >
      <span
        aria-hidden="true"
        className="mt-[5px] size-[9px] shrink-0 animate-pulse rounded-full bg-accent"
      />
      <div className="flex flex-col gap-0.5">
        <span className="text-md font-semibold">{said}</span>
        {note && <span className="text-sm text-muted">{note}</span>}
      </div>
    </div>
  );
}

function Done({ said, note }: { said: string; note?: string }) {
  return (
    <div
      role="status"
      className="mt-1.5 flex items-center gap-3 rounded-[10px] border border-good/30 bg-good/[0.08] px-4 py-3.5 dark:bg-good/10"
    >
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2.4"
        strokeLinecap="round"
        strokeLinejoin="round"
        className="size-[22px] shrink-0 text-good"
        aria-hidden="true"
      >
        <circle cx="12" cy="12" r="10" />
        <path d="M7.5 12.5l3 3 6-6.5" />
      </svg>
      <div className="flex flex-col gap-0.5">
        <span className="text-md font-semibold text-good">{said}</span>
        {note && <span className="text-sm text-ink-2">{note}</span>}
      </div>
    </div>
  );
}

function Dots({ count, at }: { count: number; at: number }) {
  return (
    <div aria-hidden="true" className="flex items-center gap-1.5">
      {Array.from({ length: count }, (_, n) => n).map((n) => (
        <span
          key={n}
          className={`h-1.5 rounded-full ${n === at ? "w-[18px] bg-accent" : "w-1.5 bg-black/20 dark:bg-white/20"}`}
        />
      ))}
    </div>
  );
}

function Footer({
  quiet,
  onQuiet,
  dots,
  strong,
  onStrong,
}: {
  quiet: string | null;
  onQuiet: () => void;
  dots: ReactNode;
  strong: string;
  onStrong: () => void;
}) {
  const main = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    main.current?.focus();
  }, [strong]);
  return (
    <div className="grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center border-t border-black/[0.08] px-6 py-4 dark:border-white/[0.08]">
      <div>
        {quiet && (
          <button
            type="button"
            onClick={onQuiet}
            className="px-1 py-2 text-md text-muted hover:underline"
          >
            {quiet}
          </button>
        )}
      </div>
      {dots}
      <button
        type="button"
        ref={main}
        onClick={onStrong}
        className="justify-self-end rounded-md bg-accent px-4 py-2.5 text-md font-semibold text-on-accent"
      >
        {strong}
      </button>
    </div>
  );
}

function noop() {}
