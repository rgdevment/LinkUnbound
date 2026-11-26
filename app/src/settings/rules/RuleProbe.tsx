import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { useSpoken, useWords } from "../../i18n";
import { saidPlainly } from "../../refusal";
import Glyph from "./Glyph";
import RuleRow, { Where } from "./RuleRow";
import { covers, FAMILY_LABEL, familyOf, type Probe, type RuleView } from "./types";

/// The resident asks the same `RuleSet`, so the answer here cannot drift from what it does.
export default function RuleProbe({
  rules,
  apps,
  onWinner,
  onCreate,
  onClose,
}: Readonly<{
  rules: RuleView[];
  apps: string[];
  onWinner: (id: string | null) => void;
  onCreate: (site: string) => void;
  onClose: () => void;
}>) {
  const t = useWords();
  const language = useSpoken();
  const [url, setUrl] = useState("");
  const [from, setFrom] = useState("");
  const [probe, setProbe] = useState<Probe | null>(null);
  const [refusal, setRefusal] = useState<string | null>(null);
  // An app whose last rule went away is no longer offered, so it is no longer asked about.
  const fromApp = apps.includes(from) ? from : "";
  const box = useRef<HTMLInputElement>(null);
  useEffect(() => box.current?.focus(), []);

  // `rules` is a dependency on purpose: a rule added or edited changes the answer.
  // biome-ignore lint/correctness/useExhaustiveDependencies: see above
  useEffect(() => {
    let current = true;
    if (!url.trim()) {
      setProbe(null);
      setRefusal(null);
      onWinner(null);
      return;
    }
    const asking = setTimeout(() => {
      void invoke<Probe>("rules_probe", { url, fromApp: fromApp || null })
        .then((answer) => {
          if (!current) return;
          setProbe(answer);
          setRefusal(null);
          onWinner(answer.winner?.id ?? null);
        })
        .catch((err: unknown) => {
          if (!current) return;
          setProbe(null);
          setRefusal(saidPlainly(language, err));
          onWinner(null);
        });
    }, 200);
    return () => {
      current = false;
      clearTimeout(asking);
    };
  }, [url, fromApp, rules, language]);

  const named = (rule: RuleView) => `${t(FAMILY_LABEL[familyOf(rule)])} · ${covers(rule)}`;
  const reason = (winner: RuleView, why: Probe["why"], next: RuleView | undefined) => {
    if (why === "origin") return t("probeWhyOrigin", named(winner));
    if (why === "narrower" && next) return t("probeWhyNarrower", named(winner), named(next));
    if (why === "earlier" && next) return t("probeWhyEarlier", named(winner), named(next));
    return t("probeWhyAlone", named(winner));
  };

  const field =
    "rounded-md border border-(--hair) bg-(--panel) px-2.5 py-1.5 text-[12px] focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-(--accent)";

  return (
    <section
      aria-label={t("probeOpen")}
      className="flex flex-col gap-2.5 rounded-lg border border-(--hair) bg-(--card) px-3.5 py-3"
    >
      <div className="flex items-center gap-2">
        <Glyph name="flask" size={15} />
        <p className="flex-1 text-[13px] font-semibold">{t("probeOpen")}</p>
        <button
          type="button"
          aria-label={t("probeClose")}
          onClick={onClose}
          className="grid h-[26px] w-[26px] place-items-center rounded-md text-(--muted) hover:bg-(--tile) hover:text-(--ink)"
        >
          <Glyph name="x" size={13} />
        </button>
      </div>
      <div className="flex flex-wrap gap-2">
        <input
          ref={box}
          aria-label={t("probeUrl")}
          placeholder="https://…"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Escape") onClose();
          }}
          className={`${field} min-w-[240px] flex-1`}
        />
        {apps.length > 0 && (
          <label className="flex items-center gap-2 text-[12px]">
            <span className="text-(--muted)">{t("kindFrom")}</span>
            <select value={fromApp} onChange={(e) => setFrom(e.target.value)} className={field}>
              <option value="">{t("probeFromAny")}</option>
              {apps.map((app) => (
                <option key={app} value={app}>
                  {app}
                </option>
              ))}
            </select>
          </label>
        )}
      </div>
      {refusal && <p className="text-[12px] text-(--danger)">{refusal}</p>}
      {probe && (
        <output className="flex flex-col gap-1 rounded-md border border-(--hair) bg-(--panel) px-3 py-2.5">
          {probe.winner ? (
            <>
              <span className="flex flex-wrap items-center gap-2 text-[13px] font-semibold">
                {probe.winner.resolved ? (
                  <>
                    {t("probeOpensIn")}
                    <span className="flex items-center gap-1.5">
                      <Where rule={probe.winner} />
                    </span>
                    {probe.winner.private && (
                      <span className="text-[12px] font-normal text-(--accent)">
                        {t("rulePrivate")}
                      </span>
                    )}
                  </>
                ) : (
                  <>
                    <span className="text-(--danger)">
                      <Glyph name="warn" size={16} />
                    </span>
                    {t("probeGone", probe.winner.browser)}
                  </>
                )}
              </span>
              <span className="block text-[12px] text-(--muted)">
                {reason(probe.winner, probe.why, probe.also[0])}
              </span>
            </>
          ) : (
            <>
              <span className="flex items-center gap-2 text-[13px] font-semibold">
                <Glyph name="info" size={16} />
                {t("probeNone")}
              </span>
              <span className="block text-[12px] text-(--muted)">
                {t("probeNoneBody")}{" "}
                <button
                  type="button"
                  onClick={() => onCreate(probe.site)}
                  className="text-(--accent) hover:underline"
                >
                  {t("probeCreate", probe.site)}
                </button>
              </span>
            </>
          )}
        </output>
      )}
      {probe?.winner && (
        <ul aria-label={t("probeMatches")} className="rounded-md border border-(--hair)">
          <RuleRow rule={probe.winner} verdict="wins" />
          {probe.also.map((rule) => (
            <RuleRow key={rule.id} rule={rule} verdict="also" />
          ))}
        </ul>
      )}
    </section>
  );
}
