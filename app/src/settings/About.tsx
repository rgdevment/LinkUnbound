import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useCallback, useEffect, useState } from "react";
import { type Key, useWords } from "../i18n";
import { Card, Line, Section, Switch } from "./parts";
import type { Ready, Underway } from "./update";

const REPO = "https://github.com/rgdevment/LinkUnbound";
const COFFEE = "https://buymeacoffee.com/rgdevment";
const COPYPASTE = "https://github.com/rgdevment/CopyPaste";

type Build = {
  version: string;
  license: string;
  repository: string;
  candidates: boolean;
  candidatesApply: boolean;
};

function External({ href, children }: { href: string; children: string }) {
  return (
    <button
      type="button"
      onClick={() => void openUrl(href).catch(noop)}
      className="shrink-0 rounded-md px-3 py-1.5 text-[11.5px] text-[#2F62D8] dark:text-[#6E9BFF]"
    >
      {children}
    </button>
  );
}

const REFUSALS = [
  "updateBusy",
  "updateGone",
  "updateNotHere",
  "updateElsewhere",
  "updateStopped",
  "updateFailed",
] as const;

function said(t: (key: Key, ...values: string[]) => string, problem: unknown): string {
  const raw = String(problem).replace(/^Error:\s*/, "");
  return (REFUSALS as readonly string[]).includes(raw) ? t(raw as Key) : raw;
}

function Pip({ ok }: { ok?: boolean }) {
  return (
    <span
      aria-hidden="true"
      className={`h-[7px] w-[7px] shrink-0 rounded-full ${
        ok ? "bg-[#1E7A52] dark:bg-[#4CC38A]" : "bg-[#2F62D8] dark:bg-[#6E9BFF]"
      }`}
    />
  );
}

function Offer({
  ready,
  step,
  onProblem,
  onSettled,
  onLook,
}: {
  ready: Ready | null;
  step: Underway | null;
  onProblem: (why: string | null) => void;
  onSettled: () => void;
  onLook: (nowPlease?: boolean) => Promise<unknown>;
}) {
  const t = useWords();
  const [looking, setLooking] = useState(false);
  const [asked, setAsked] = useState(false);

  const lookAgain = () => {
    setLooking(true);
    onProblem(null);
    onLook(true)
      .catch((e: unknown) => onProblem(said(t, e)))
      .finally(() => setLooking(false));
  };

  const newer = ready;

  if (!newer) {
    return (
      <div className="flex items-center gap-2.5 rounded-lg border border-black/[0.08] bg-black/[0.02] px-3.5 py-3 dark:border-white/[0.08] dark:bg-white/[0.02]">
        <Pip ok />
        <span className="flex-1 text-[11.5px]">{looking ? t("lookingNow") : t("lookNowNone")}</span>
        <button
          type="button"
          disabled={looking}
          onClick={lookAgain}
          className="shrink-0 rounded-md border border-black/[0.12] px-3 py-1.5 text-[11.5px] disabled:opacity-40 dark:border-white/[0.12]"
        >
          {t("lookNow")}
        </button>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-3 rounded-lg border border-[#2F62D8]/25 bg-[#2F62D8]/[0.06] px-3.5 py-3 dark:border-[#6E9BFF]/25 dark:bg-[#6E9BFF]/[0.08]">
      <Pip />
      <div className="min-w-0 flex-1" aria-live="polite">
        <p className="text-[12.5px] font-semibold">{t("updateThere", newer.version)}</p>
        {step ? (
          step.stage === "installing" ? (
            <p className="mt-0.5 text-[11.5px] text-neutral-600 dark:text-[#98A0B4]">
              {t(newer.route === "store" ? "updateInstallingStore" : "updateInstalling")}
            </p>
          ) : (
            <>
              <p className="mt-0.5 text-[11.5px] text-neutral-600 dark:text-[#98A0B4]">
                {t("updateGetting", `${step.far} %`)}
              </p>
              <span
                role="progressbar"
                aria-label={t("updateInstall")}
                aria-valuenow={step.far}
                aria-valuemin={0}
                aria-valuemax={100}
                className="mt-1.5 block h-1 overflow-hidden rounded-full bg-black/[0.08] dark:bg-white/[0.12]"
              >
                <span
                  className="block h-full rounded-full bg-[#2F62D8] transition-[width] dark:bg-[#6E9BFF]"
                  style={{ width: `${step.far}%` }}
                />
              </span>
            </>
          )
        ) : asked ? (
          <p className="mt-0.5 text-[11.5px] text-neutral-600 dark:text-[#98A0B4]">
            {t("updateStarting")}
          </p>
        ) : (
          <p className="mt-0.5 text-[11.5px] text-neutral-600 dark:text-[#98A0B4]">
            {newer.route === "store" ? t("updateStore") : t("updateAsk")}
          </p>
        )}
      </div>
      {!step && newer.installs && (
        <button
          type="button"
          disabled={asked}
          onClick={() => {
            setAsked(true);
            onProblem(null);
            invoke("update_install")
              // Every other route takes the process with it, and a button handed back there would
              // only offer an update to a copy on its way out.
              .then(() => {
                if (newer.route !== "store") return;
                setAsked(false);
                onSettled();
                lookAgain();
              })
              .catch((e: unknown) => {
                setAsked(false);
                // The progress stops arriving but never says so, and the button stays hidden
                // behind it: a cancelled store update could not be tried again.
                onSettled();
                onProblem(said(t, e));
              });
          }}
          className="shrink-0 rounded-md bg-[#2F62D8] px-3 py-1.5 text-[11.5px] font-medium text-white disabled:opacity-50 dark:bg-[#6E9BFF] dark:text-[#12141B]"
        >
          {t("updateInstall")}
        </button>
      )}
    </div>
  );
}

/// Not offered to a copy the store keeps: the store carries no candidates, so the choice would
/// change nothing there.
function Candidates({
  on,
  onChange,
  onProblem,
  onAsked,
}: {
  on: boolean;
  onChange: (next: boolean) => void;
  onProblem: (why: string | null) => void;
  onAsked: () => Promise<unknown>;
}) {
  const t = useWords();

  return (
    <Card>
      <Line title={t("candidatesTitle")} note={t("candidatesNote")}>
        <Switch
          on={on}
          label={t("candidatesTitle")}
          onChange={(wants) => {
            onChange(wants);
            onProblem(null);
            invoke("update_candidates", { wants })
              .then(onAsked)
              .catch((e: unknown) => {
                onChange(!wants);
                onProblem(said(t, e));
              });
          }}
        />
      </Line>
    </Card>
  );
}

export default function About({
  ready,
  step,
  onSettled,
  onLook,
}: {
  ready: Ready | null;
  step: Underway | null;
  onSettled: () => void;
  onLook: (nowPlease?: boolean) => Promise<unknown>;
}) {
  const t = useWords();
  const [build, setBuild] = useState<Build | null>(null);
  const [problem, setProblem] = useState<string | null>(null);

  const look = useCallback(() => {
    invoke<Build>("about").then(setBuild).catch(noop);
  }, []);

  useEffect(look, [look]);

  return (
    <>
      <div className="flex items-center gap-3.5 rounded-lg border border-black/[0.08] bg-black/[0.015] px-4 py-3.5 dark:border-white/[0.08] dark:bg-white/[0.02]">
        <span
          aria-hidden="true"
          className="grid h-11 w-11 shrink-0 place-items-center rounded-[10px] bg-[#2F62D8] text-[17px] font-semibold text-white dark:bg-[#6E9BFF] dark:text-[#12141B]"
        >
          L
        </span>
        <div className="min-w-0 flex-1">
          <p className="text-[13.5px] font-semibold">LinkUnbound</p>
          <p className="mt-0.5 text-[11.5px] text-neutral-500 tabular-nums dark:text-[#8B92A1]">
            {build ? `${build.version} · ${build.license}` : "—"}
          </p>
          <p className="mt-1 text-[11.5px] text-neutral-500 dark:text-[#8B92A1]">
            {t("aboutTagline")}
          </p>
        </div>
      </div>

      {problem && (
        <p
          role="alert"
          className="rounded-md bg-[#C0362F]/10 px-3 py-2 text-[11.5px] text-[#C0362F] dark:bg-[#FF8A85]/10 dark:text-[#FF8A85]"
        >
          {problem}
        </p>
      )}

      <Offer
        ready={ready}
        step={step}
        onProblem={setProblem}
        onSettled={onSettled}
        onLook={onLook}
      />

      {build?.candidatesApply && (
        <Candidates
          on={build.candidates}
          onChange={(next) => setBuild((was) => (was ? { ...was, candidates: next } : was))}
          onProblem={setProblem}
          onAsked={onLook}
        />
      )}

      <Section title={t("aboutProject")}>
        <Card>
          <Line title={t("aboutSource")} note={t("aboutSourceNote")}>
            <External href={build?.repository ?? REPO}>{t("open")}</External>
          </Line>
          <Line title={t("aboutLicense")} note={t("aboutLicenseNote")}>
            <External href={`${REPO}/blob/main/LICENSE`}>{t("open")}</External>
          </Line>
          <Line title={t("aboutIssue")} note={t("aboutIssueNote")}>
            <External href={`${REPO}/issues`}>{t("open")}</External>
          </Line>
        </Card>
      </Section>

      <Section title={t("aboutSponsor")}>
        <Card>
          <Line title={t("aboutSponsor")} note={t("aboutSponsorNote")}>
            <External href={COFFEE}>{t("open")}</External>
          </Line>
        </Card>
      </Section>

      <Section title={t("aboutOtherTools")}>
        <Card>
          <Line title={t("aboutCopyPaste")} note={t("aboutCopyPasteNote")}>
            <External href={COPYPASTE}>{t("open")}</External>
          </Line>
        </Card>
      </Section>
    </>
  );
}

function noop() {}
