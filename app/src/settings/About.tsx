import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useCallback, useEffect, useState } from "react";
import { type Key, useSpoken, useWords } from "../i18n";
import { offerMoved, saidPlainly } from "../refusal";
import { CloudOff, Code, Gift, Key as KeyIcon } from "./Icons";
import { Card, Line, Switch } from "./parts";
import Star from "./Star";
import type { Ready, Underway } from "./update";

const REPO = "https://github.com/rgdevment/LinkUnbound";
const COFFEE = "https://buymeacoffee.com/rgdevment";
const COPYPASTE = "https://github.com/rgdevment/CopyPaste";
const TISTY = "https://github.com/rgdevment/Tisty";
const SPONSOR = "https://github.com/sponsors/rgdevment";
const RATING = "ms-windows-store://review/?ProductId=9N9F7C8Q43KC";
const ALTERNATIVETO = "https://alternativeto.net/software/linkunbound/";

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
    <button type="button" onClick={() => void openUrl(href).catch(noop)}>
      {children}
    </button>
  );
}

function Rule({ said }: { said: string }) {
  return <div className="rule">{said}</div>;
}

function Badge({ said, children }: { said: string; children: React.ReactNode }) {
  return (
    <span className="badge">
      {children}
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
    <button type="button" onClick={onPick} className={wide ? "give wide" : "give"}>
      <svg viewBox="0 0 16 16" aria-hidden="true">
        {children}
      </svg>
      <span className="min-w-0">
        <b>{said}</b>
        <span>{where}</span>
      </span>
    </button>
  );
}

/// The mark is the app's own initial rather than a copied icon: the picture lives in the other
/// project, and a missing file here would leave a broken image in a row that is only a pointer.
function Tool({
  mark,
  hue,
  said,
  note,
  onPick,
}: {
  mark: string;
  hue: string;
  said: string;
  note: string;
  onPick: () => void;
}) {
  return (
    <button type="button" className="tool" onClick={onPick}>
      <span className="ico" aria-hidden="true" style={{ background: hue }}>
        {mark}
      </span>
      <span className="min-w-0 flex-1">
        <b>{said}</b>
        <span>{note}</span>
      </span>
    </button>
  );
}

function Pip({ ok }: { ok?: boolean }) {
  return <span aria-hidden="true" className={ok ? "pip ok" : "pip"} />;
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
      <div className="newer">
        <Pip ok />
        <span className="grow">
          <b>{looking ? t("lookingNow") : t("lookNowNone")}</b>
        </span>
        <button type="button" disabled={looking} onClick={lookAgain} className="mild">
          {t("lookNow")}
        </button>
      </div>
    );
  }

  return (
    <div className="newer there">
      <Pip />
      <div className="grow" aria-live="polite">
        <b>{t("updateThere", newer.version)}</b>
        {step ? (
          step.stage === "installing" ? (
            <span>{t(newer.route === "store" ? "updateInstallingStore" : "updateInstalling")}</span>
          ) : (
            <>
              <span>{t("updateGetting", `${step.far} %`)}</span>
              <span
                role="progressbar"
                aria-label={t("updateInstall")}
                aria-valuenow={step.far}
                aria-valuemin={0}
                aria-valuemax={100}
                className="bar"
              >
                <span style={{ width: `${step.far}%` }} />
              </span>
            </>
          )
        ) : asked ? (
          <span>{t("updateStarting")}</span>
        ) : (
          <span>{t(howItInstalls(newer))}</span>
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
          className="strong"
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
  const [saved, setSaved] = useState<string | null>(null);

  const report = () => {
    setProblem(null);
    invoke<string>("maintenance_report")
      .then(setSaved)
      .catch((e: unknown) => setProblem(saidPlainly(language, e)));
  };

  const look = useCallback(() => {
    setTrouble(null);
    invoke<Build>("about")
      .then(setBuild)
      .catch((e: unknown) => setTrouble(saidPlainly(language, e)));
  }, [language]);

  useEffect(look, [look]);

  const store = build?.keptByTheStore ?? false;
  const offerStar = !starring || store;
  const offerRate = store && !starring;
  const lopsided = (offerStar ? 1 : 0) + (offerRate ? 1 : 0) === 1;

  return (
    <>
      <div className="brow">
        <h1>LinkUnbound</h1>
        <span className="line2">
          <span>{build?.version ?? "—"}</span>
          <i />
          <span>{build?.license ?? "—"}</span>
        </span>
      </div>

      <div className="what-is">
        <p>{t("aboutWhat")}</p>
        <p>{t("aboutPrivacy")}</p>
        <div className="badges">
          <Badge said={t("badgeLocal")}>
            <KeyIcon />
          </Badge>
          <Badge said={t("badgeOpen")}>
            <Code />
          </Badge>
          <Badge said={t("badgeFree")}>
            <Gift />
          </Badge>
          <Badge said={t("badgeQuiet")}>
            <CloudOff />
          </Badge>
        </div>
      </div>

      {trouble && (
        <div className="alarm">
          <p role="alert">
            {t("aboutFailed")} · {trouble}
          </p>
          <button type="button" onClick={look} className="mild" style={{ marginTop: 6 }}>
            {t("tryAgain")}
          </button>
        </div>
      )}

      {problem && (
        <p role="alert" className="alarm">
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

      {starring && <Star store={store} onSettled={onStarSettled} />}

      <Rule said={t("aboutSponsorSection")} />
      <p className="quiet">{t("aboutSupportWhy")}</p>
      <div className="gives">
        {offerStar && (
          <Gives
            wide={lopsided}
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
        {offerRate && (
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
        hue="#6f4bd8"
        said={t("aboutTisty")}
        note={t("aboutTistyNote")}
        onPick={() => void openUrl(TISTY).catch(noop)}
      />
      <Tool
        mark="CP"
        hue="#1e7a52"
        said={t("aboutCopyPaste")}
        note={t("aboutCopyPasteNote")}
        onPick={() => void openUrl(COPYPASTE).catch(noop)}
      />

      <Rule said={t("troubleSection")} />
      <p className="trouble">
        {t("reportWhat")} <b>{t("reportStays")}</b> {t("reportPick")}
      </p>
      <div className="row">
        <button type="button" onClick={report} className="mild">
          {t("reportGo")}
        </button>
        <button
          type="button"
          onClick={() => void openUrl(`${REPO}/issues`).catch(noop)}
          className="mild"
        >
          {t("aboutIssue")}
        </button>
      </div>
      {saved && <p className="saved">{t("reportSaved", saved)}</p>}

      <div className="links">
        <External href={build?.repository ?? REPO}>{t("aboutRepo")}</External>
        <External href={ALTERNATIVETO}>AlternativeTo</External>
        <External href={`${REPO}/blob/main/PRIVACY.md`}>{t("aboutPrivacyLink")}</External>
        <External href={`${REPO}/blob/main/THIRD-PARTY-BUNDLED.md`}>{t("aboutNotices")}</External>
      </div>
    </>
  );
}

function noop() {}
