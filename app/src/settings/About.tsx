import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useCallback, useEffect, useState } from "react";
import { type Key, useSpoken, useWords } from "../i18n";
import { offerMoved, saidPlainly } from "../refusal";
import { Card, Line, Switch } from "./parts";
import Star from "./Star";
import type { Ready, Underway } from "./update";

const REPO = "https://github.com/rgdevment/LinkUnbound";
const COFFEE = "https://buymeacoffee.com/rgdevment";
const COPYPASTE = "https://github.com/rgdevment/CopyPaste";
const TISTY = "https://github.com/rgdevment/Tisty";
const SPONSOR = "https://github.com/sponsors/rgdevment";
const RATING = "ms-windows-store://review/?ProductId=9N9F7C8Q43KC";

type Build = {
  version: string;
  license: string;
  repository: string;
  candidates: boolean;
  candidatesApply: boolean;
  keptByTheStore: boolean;
};

/// What happens on «Actualizar» — or, when there is no button, what the person has to do instead:
/// a copy the store keeps but did not sell cannot be installed from here, and one running from
/// the disk image it was downloaded as cannot replace itself.
function howItInstalls(newer: Ready): Key {
  if (newer.installs) return newer.route === "store" ? "updateStore" : "updateAsk";
  return newer.route === "store" ? "updateStoreByHand" : "updateFromMount";
}

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

function Rule({ said }: { said: string }) {
  return (
    <div className="mt-6 mb-2 flex items-center gap-2.5 text-[11.5px] font-semibold uppercase tracking-[0.05em] text-neutral-500 dark:text-[#8B92A1]">
      <span>{said}</span>
      <span className="h-px flex-1 bg-black/[0.08] dark:bg-white/[0.08]" />
    </div>
  );
}

function Badge({ said }: { said: string }) {
  return (
    <span className="rounded-full border border-black/[0.08] px-2.5 py-1 text-[11.5px] text-neutral-600 dark:border-white/[0.08] dark:text-[#A8AEBC]">
      {said}
    </span>
  );
}

function Gives({
  said,
  where,
  wide,
  onPick,
  children,
}: {
  said: string;
  where: string;
  wide?: boolean;
  onPick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onPick}
      className={`flex items-center gap-2.5 rounded-[10px] border border-black/[0.08] px-3 py-2.5 text-left hover:bg-black/[0.03] dark:border-white/[0.08] dark:hover:bg-white/[0.04] ${
        wide ? "col-span-2" : ""
      }`}
    >
      <svg viewBox="0 0 16 16" aria-hidden="true" className="h-[17px] w-[17px] shrink-0">
        {children}
      </svg>
      <span className="min-w-0">
        <span className="block text-[12.5px] font-medium">{said}</span>
        <span className="block truncate text-[11.5px] text-neutral-500 dark:text-[#8B92A1]">
          {where}
        </span>
      </span>
    </button>
  );
}

/// The mark is the app's own initial rather than a copied icon: the picture lives in the other
/// project, and a missing file here would leave a broken image in a row that is only a pointer.
function Tool({
  mark,
  tone,
  said,
  note,
  onPick,
}: {
  mark: string;
  tone: string;
  said: string;
  note: string;
  onPick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onPick}
      className="mb-2 flex w-full items-start gap-3 rounded-[10px] border border-black/[0.08] px-3.5 py-3 text-left hover:bg-black/[0.03] dark:border-white/[0.08] dark:hover:bg-white/[0.04]"
    >
      <span
        aria-hidden="true"
        className={`mt-px grid h-6 w-6 shrink-0 place-items-center rounded-md text-[11px] font-semibold text-white ${tone}`}
      >
        {mark}
      </span>
      <span className="min-w-0 flex-1">
        <span className="block text-[13px] font-semibold">{said}</span>
        <span className="mt-0.5 block text-[12.5px] leading-relaxed text-neutral-500 dark:text-[#8B92A1]">
          {note}
        </span>
      </span>
      <span aria-hidden="true" className="mt-0.5 text-[13px] text-neutral-400 dark:text-[#6B7280]">
        ↗
      </span>
    </button>
  );
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
  const language = useSpoken();
  const [looking, setLooking] = useState(false);
  const [asked, setAsked] = useState(false);

  const lookAgain = () => {
    setLooking(true);
    onProblem(null);
    onLook(true)
      .catch((e: unknown) => onProblem(saidPlainly(language, e)))
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
            {t(howItInstalls(newer))}
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
                onSettled();
                onProblem(saidPlainly(language, e));
                if (offerMoved(e)) {
                  onLook().catch(() => {});
                }
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
  onAsked: () => void;
}) {
  const t = useWords();
  const language = useSpoken();

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
                onProblem(saidPlainly(language, e));
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
  starring,
  onStarSettled,
  onSettled,
  onLook,
}: {
  ready: Ready | null;
  step: Underway | null;
  starring: boolean;
  onStarSettled: () => void;
  onSettled: () => void;
  onLook: (nowPlease?: boolean) => Promise<unknown>;
}) {
  const t = useWords();
  const language = useSpoken();
  const [build, setBuild] = useState<Build | null>(null);
  const [trouble, setTrouble] = useState<string | null>(null);
  const [problem, setProblem] = useState<string | null>(null);

  const look = useCallback(() => {
    setTrouble(null);
    invoke<Build>("about")
      .then(setBuild)
      .catch((e: unknown) => setTrouble(saidPlainly(language, e)));
  }, [language]);

  useEffect(look, [look]);

  return (
    <>
      <div className="flex items-center gap-3.5">
        <span
          aria-hidden="true"
          className="grid h-[52px] w-[52px] shrink-0 place-items-center rounded-[10px] bg-[#2F62D8] text-[21px] font-semibold text-white dark:bg-[#6E9BFF] dark:text-[#12141B]"
        >
          L
        </span>
        <span className="min-w-0">
          <h2 className="text-[21px] font-semibold tracking-[-0.015em]">LinkUnbound</h2>
          <span className="mt-px flex items-center gap-2 text-[11.5px] text-neutral-500 tabular-nums dark:text-[#8B92A1]">
            <span>{build?.version ?? "—"}</span>
            <span
              aria-hidden="true"
              className="h-[3px] w-[3px] rounded-full bg-black/20 dark:bg-white/20"
            />
            <span>{build?.license ?? "—"}</span>
          </span>
        </span>
      </div>

      <div className="mt-4 rounded-[10px] border border-black/[0.08] bg-black/[0.015] px-4 py-3.5 dark:border-white/[0.08] dark:bg-white/[0.02]">
        <p className="text-[12.5px] leading-relaxed text-neutral-600 dark:text-[#A8AEBC]">
          {t("aboutWhat")}
        </p>
        <p className="mt-1 text-[12.5px] leading-relaxed text-neutral-500 dark:text-[#8B92A1]">
          {t("aboutPrivacy")}
        </p>
        <div className="mt-3 flex flex-wrap gap-1.5">
          <Badge said={t("badgeLocal")} />
          <Badge said={t("badgeOpen")} />
          <Badge said={t("badgeFree")} />
          <Badge said={t("badgeQuiet")} />
        </div>
      </div>

      {trouble && (
        <div className="rounded-md bg-[#C0362F]/10 px-3 py-2 dark:bg-[#FF8A85]/10">
          <p role="alert" className="text-[11.5px] text-[#C0362F] dark:text-[#FF8A85]">
            {t("aboutFailed")} · {trouble}
          </p>
          <button
            type="button"
            onClick={look}
            className="mt-1.5 rounded-md border border-black/[0.12] px-2.5 py-1 text-[11.5px] dark:border-white/[0.12]"
          >
            {t("tryAgain")}
          </button>
        </div>
      )}

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

      {starring && <Star onSettled={onStarSettled} />}

      <Rule said={t("aboutSponsorSection")} />
      <p className="text-[13px] leading-relaxed text-neutral-600 dark:text-[#A8AEBC]">
        {t("aboutSupportWhy")}
      </p>
      <div className="mt-2.5 grid grid-cols-2 gap-2.5">
        {!starring && (
          <Gives
            wide={!build?.keptByTheStore}
            said={t("aboutStar")}
            where="github.com/rgdevment/LinkUnbound"
            onPick={() => void openUrl(REPO).catch(noop)}
          >
            <path
              fill="#e3b341"
              d="M8 1.2l2.1 4.3 4.7.7-3.4 3.3.8 4.7L8 12l-4.2 2.2.8-4.7L1.2 6.2l4.7-.7L8 1.2z"
            />
          </Gives>
        )}
        {build?.keptByTheStore && (
          <Gives
            said={t("aboutRate")}
            where="Microsoft Store"
            onPick={() => void openUrl(RATING).catch(noop)}
          >
            <path
              fill="#0078d4"
              d="M2 3h12a1 1 0 0 1 1 1v7a1 1 0 0 1-1 1H6l-4 3V4a1 1 0 0 1 1-1Z"
            />
          </Gives>
        )}
        <Gives
          said={t("aboutGitHubSponsor")}
          where="github.com/sponsors"
          onPick={() => void openUrl(SPONSOR).catch(noop)}
        >
          <path
            fill="#db61a2"
            d="M8 14.25 6.84 13.2C2.72 9.47 0 7.01 0 4.5 0 2.42 1.57 1 3.5 1c1.1 0 2.16.51 2.84 1.32h1.32C8.34 1.51 9.4 1 10.5 1 12.43 1 14 2.42 14 4.5c0 2.51-2.72 4.97-6.84 8.7L8 14.25Z"
          />
        </Gives>
        <Gives
          said={t("aboutSponsor")}
          where="buymeacoffee.com"
          onPick={() => void openUrl(COFFEE).catch(noop)}
        >
          <path
            fill="#c8892a"
            d="M2 5h9v5a3 3 0 0 1-3 3H5a3 3 0 0 1-3-3V5Zm10 0h1.5A2.5 2.5 0 0 1 16 7.5 2.5 2.5 0 0 1 13.5 10H12V5ZM2 14h9v1H2v-1Z"
          />
        </Gives>
      </div>

      <Rule said={t("aboutOtherTools")} />
      <Tool
        mark="T"
        tone="bg-[#6f4bd8]"
        said={t("aboutTisty")}
        note={t("aboutTistyNote")}
        onPick={() => void openUrl(TISTY).catch(noop)}
      />
      <Tool
        mark="CP"
        tone="bg-[#1E7A52] dark:bg-[#2E9B6B]"
        said={t("aboutCopyPaste")}
        note={t("aboutCopyPasteNote")}
        onPick={() => void openUrl(COPYPASTE).catch(noop)}
      />

      <div className="mt-4 flex flex-wrap gap-2">
        <External href={build?.repository ?? REPO}>{t("aboutRepo")}</External>
        <External href={`${REPO}/issues`}>{t("aboutIssue")}</External>
      </div>
    </>
  );
}

function noop() {}
