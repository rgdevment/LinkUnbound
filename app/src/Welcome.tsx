import { invoke } from "@tauri-apps/api/core";
import { type ReactNode, useCallback, useEffect, useRef, useState } from "react";
import bear from "./assets/linkunbound.svg";
import { type Key, type Language, Speaking, spoken, useSpoken, useWords } from "./i18n";
import { saidPlainly } from "./refusal";
import { chooseUs, type SystemState } from "./system";

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
  const [state, setState] = useState<SystemState | null>(null);

  // The step that hands the links over is left out for someone who already did.
  useEffect(() => {
    void invoke<SystemState>("system_state")
      .then((found) => {
        setState(found);
        setSteps(found.is_default ? ["hello", "try"] : ["hello", "links", "try"]);
      })
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
    <div className="flex h-full flex-col bg-white text-[#16181D] dark:bg-[#191A1F] dark:text-[#F0F1F4]">
      {step === "hello" && <Hello dots={dots} onSkip={close} onNext={next} />}
      {step === "links" && state && (
        <Links dots={dots} state={state} onState={setState} onNext={next} />
      )}
      {step === "try" && (
        <Try dots={dots} handing={state?.is_default ?? true} shortcut={shortcut} onDone={close} />
      )}
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
        <h1 className="text-[24px] font-semibold">{t("welcomeHelloTitle")}</h1>
        <p className="max-w-[460px] text-[14px] leading-relaxed text-neutral-600 dark:text-[#98A0B4]">
          {t("welcomeHelloBody")}
        </p>
        <ul className="mt-1 flex flex-wrap justify-center gap-2">
          {(["welcomePillLocal", "welcomePillAccount", "welcomePillBrowsers"] as Key[]).map(
            (pill) => (
              <li
                key={pill}
                className="rounded-full bg-[#2F62D8]/10 px-3 py-1 text-[12px] text-[#24489F] dark:bg-[#6E9BFF]/15 dark:text-[#9DBBFF]"
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

function Links({
  dots,
  state,
  onState,
  onNext,
}: {
  dots: ReactNode;
  state: SystemState;
  onState: (next: SystemState) => void;
  onNext: () => void;
}) {
  const t = useWords();
  const language = useSpoken();
  const [progress, setProgress] = useState<Progress>(state.is_default ? "done" : "idle");
  const [problem, setProblem] = useState<string | null>(null);

  // Windows answers in its own window, so the only way to hear it is to keep asking.
  useEffect(() => {
    if (progress !== "waiting") return;
    const asking = setInterval(() => {
      void invoke<SystemState>("system_state")
        .then((found) => {
          onState(found);
          if (found.is_default) setProgress("done");
        })
        .catch(noop);
    }, EVERY_SECOND);
    return () => clearInterval(asking);
  }, [progress, onState]);

  useEffect(() => {
    if (progress !== "done") return;
    const onwards = setTimeout(onNext, LONG_ENOUGH_TO_READ);
    return () => clearTimeout(onwards);
  }, [progress, onNext]);

  const choose = () => {
    setProblem(null);
    chooseUs(state)
      .then((found) => {
        onState(found);
        setProgress(found.is_default ? "done" : "waiting");
      })
      .catch((e: unknown) => setProblem(saidPlainly(language, e)));
  };

  return (
    <>
      <Page over={t("welcomeLinkOver")} title={t("welcomeLinkTitle")} body={t("welcomeLinkBody")}>
        <ol
          className={`mt-1 flex flex-col gap-2 text-[13.5px] ${progress === "done" ? "text-neutral-400 dark:text-[#646B7C]" : ""}`}
        >
          {(["welcomeLinkStep1", "welcomeLinkStep2", "welcomeLinkStep3"] as Key[]).map(
            (line, n) => (
              <li key={line} className="flex items-baseline gap-2.5">
                <span className="font-semibold text-[#2F62D8] dark:text-[#6E9BFF]">{n + 1}</span>
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
          <p role="alert" className="text-[12px] text-[#C0362F] dark:text-[#FF8A85]">
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
        onStrong={progress === "done" ? onNext : choose}
      />
    </>
  );
}

function Try({
  dots,
  handing,
  shortcut,
  onDone,
}: {
  dots: ReactNode;
  handing: boolean;
  shortcut: string | null;
  onDone: () => void;
}) {
  const t = useWords();
  const [progress, setProgress] = useState<Progress>("idle");

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

  const go = () => {
    void invoke("welcome_try")
      .then(() => setProgress((was) => (was === "done" ? was : "waiting")))
      .catch(noop);
  };

  return (
    <>
      <Page over={t("welcomeTryOver")} title={t("welcomeTryTitle")} body={t("welcomeTryBody")}>
        <div className="flex justify-center py-2">
          <button
            type="button"
            onClick={go}
            className="flex items-center gap-2 rounded-lg border border-[#2F62D8]/25 bg-[#2F62D8]/[0.06] px-4 py-3 text-[14px] text-[#24489F] hover:bg-[#2F62D8]/10 dark:border-[#6E9BFF]/30 dark:bg-[#6E9BFF]/10 dark:text-[#9DBBFF]"
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
            rgdevment.com
          </button>
        </div>
        {!handing && progress !== "done" && (
          <p className="text-[12px] text-[#8A4A10] dark:text-[#E9A05C]">{t("welcomeTryNotYet")}</p>
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
      <span className="text-[11px] font-semibold tracking-[0.08em] text-neutral-500 uppercase dark:text-[#8B92A1]">
        {over}
      </span>
      <h1 className="text-[22px] font-semibold">{title}</h1>
      <p className="max-w-[500px] text-[14px] leading-relaxed text-neutral-600 dark:text-[#98A0B4]">
        {body}
      </p>
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
        className="mt-[5px] size-[9px] shrink-0 animate-pulse rounded-full bg-[#2F62D8] dark:bg-[#6E9BFF]"
      />
      <div className="flex flex-col gap-0.5">
        <span className="text-[13.5px] font-semibold">{said}</span>
        {note && <span className="text-[12px] text-neutral-500 dark:text-[#8B92A1]">{note}</span>}
      </div>
    </div>
  );
}

function Done({ said, note }: { said: string; note?: string }) {
  return (
    <div
      role="status"
      className="mt-1.5 flex items-center gap-3 rounded-[10px] border border-[#1E7A52]/30 bg-[#1E7A52]/[0.08] px-4 py-3.5 dark:border-[#4CC38A]/30 dark:bg-[#4CC38A]/10"
    >
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2.4"
        strokeLinecap="round"
        strokeLinejoin="round"
        className="size-[22px] shrink-0 text-[#1E7A52] dark:text-[#4CC38A]"
        aria-hidden="true"
      >
        <circle cx="12" cy="12" r="10" />
        <path d="M7.5 12.5l3 3 6-6.5" />
      </svg>
      <div className="flex flex-col gap-0.5">
        <span className="text-[13.5px] font-semibold text-[#155C3D] dark:text-[#4CC38A]">
          {said}
        </span>
        {note && <span className="text-[12px] text-[#2E5F47] dark:text-[#98A0B4]">{note}</span>}
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
          className={`h-1.5 rounded-full ${n === at ? "w-[18px] bg-[#2F62D8] dark:bg-[#6E9BFF]" : "w-1.5 bg-black/20 dark:bg-white/20"}`}
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
            className="px-1 py-2 text-[13px] text-neutral-500 hover:underline dark:text-[#8B92A1]"
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
        className="justify-self-end rounded-md bg-[#2F62D8] px-4 py-2.5 text-[13px] font-semibold text-white dark:bg-[#6E9BFF] dark:text-[#12141B]"
      >
        {strong}
      </button>
    </div>
  );
}

function noop() {}
