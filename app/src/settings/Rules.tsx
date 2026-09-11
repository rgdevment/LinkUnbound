import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { type Key, useWords } from "../i18n";

type RuleView = {
  id: string;
  kind: "any" | "url" | "host" | "site";
  covers: string;
  browser: string;
  profile: string | null;
  icon: string | null;
  private: boolean;
  source_app: string | null;
  resolved: boolean;
};

const KIND: Record<RuleView["kind"], Key> = {
  any: "kindAny",
  url: "kindUrl",
  host: "kindHost",
  site: "kindSite",
};

function covers(rule: RuleView): string {
  if (rule.source_app && rule.kind === "any") return rule.source_app;
  return rule.covers;
}

export default function Rules() {
  const t = useWords();
  const [rules, setRules] = useState<RuleView[] | null>(null);
  const [problem, setProblem] = useState<string | null>(null);

  const load = useCallback((command: string, args: Record<string, unknown> = {}) => {
    void invoke<RuleView[]>(command, args)
      .then((next) => {
        setRules(next);
        setProblem(null);
      })
      .catch((e: unknown) => setProblem(String(e)));
  }, []);

  useEffect(() => load("rules_list"), [load]);

  const move = (index: number, step: number) => {
    if (!rules) return;
    const next = [...rules];
    const target = index + step;
    if (target < 0 || target >= next.length) return;
    [next[index], next[target]] = [next[target], next[index]];
    load("rules_reorder", { ids: next.map((r) => r.id) });
  };

  if (problem) {
    return (
      <p className="rounded-md bg-[#C0362F]/10 px-3 py-2 text-[11.5px] text-[#C0362F] dark:bg-[#FF8A85]/10 dark:text-[#FF8A85]">
        {problem}
      </p>
    );
  }

  if (rules === null) return null;

  if (rules.length === 0) {
    return (
      <div className="rounded-lg border border-dashed border-black/[0.12] px-4 py-7 text-center dark:border-white/[0.12]">
        <p className="text-[12.5px] font-semibold">{t("rulesEmptyTitle")}</p>
        <p className="mt-1 text-[12px] text-neutral-500 dark:text-[#8B92A1]">
          {t("rulesEmptyBody")}
        </p>
      </div>
    );
  }

  return (
    <section className="flex flex-col gap-2">
      <h2 className="text-[11px] font-semibold tracking-[0.09em] text-neutral-400 uppercase dark:text-[#646B7C]">
        {rules.length === 1 ? t("rulesOne") : t("rulesMany", String(rules.length))}
      </h2>

      <ul className="overflow-hidden rounded-lg border border-black/[0.08] dark:border-white/[0.08]">
        {rules.map((rule, i) => (
          <li
            key={rule.id}
            className="group flex items-center gap-3 border-black/[0.08] bg-black/[0.015] px-3 py-2.5 not-first:border-t hover:bg-black/[0.04] dark:border-white/[0.08] dark:bg-white/[0.02] dark:hover:bg-white/[0.05]"
          >
            <span
              className={`w-[70px] shrink-0 rounded px-1.5 py-0.5 text-center text-[10px] tracking-[0.03em] uppercase ${
                rule.source_app
                  ? "bg-[#A85B14]/[0.14] text-[#A85B14] dark:bg-[#E9A05C]/[0.16] dark:text-[#E9A05C]"
                  : rule.kind === "url"
                    ? "bg-[#2F62D8]/[0.11] text-[#2F62D8] dark:bg-[#6E9BFF]/[0.16] dark:text-[#6E9BFF]"
                    : "bg-black/[0.055] text-neutral-500 dark:bg-white/[0.07] dark:text-[#8B92A1]"
              }`}
            >
              {rule.source_app ? t("kindFrom") : t(KIND[rule.kind])}
            </span>

            <span className="min-w-0 flex-1 truncate text-[12.5px]">{covers(rule)}</span>
            <span className="shrink-0 text-[11px] text-neutral-400 dark:text-[#646B7C]">→</span>

            <span className="flex shrink-0 items-center gap-1.5 text-[12px]">
              {rule.icon ? (
                <img src={rule.icon} alt="" className="h-4 w-4" />
              ) : (
                <span className="h-4 w-4 rounded bg-black/10 dark:bg-white/10" />
              )}
              <span className={rule.resolved ? "" : "line-through opacity-60"}>{rule.browser}</span>
              {rule.profile && (
                <span className="rounded bg-black/[0.055] px-1.5 py-px text-[10.5px] text-neutral-500 dark:bg-white/[0.07] dark:text-[#8B92A1]">
                  {rule.profile}
                </span>
              )}
            </span>

            {rule.private && (
              <span className="shrink-0 text-[10.5px] text-[#2F62D8] dark:text-[#6E9BFF]">
                {t("rulePrivate")}
              </span>
            )}

            <span className="flex shrink-0 gap-px opacity-0 transition group-hover:opacity-100 focus-within:opacity-100">
              <button
                type="button"
                aria-label={t("ruleUp", covers(rule))}
                disabled={i === 0}
                onClick={() => move(i, -1)}
                className="grid h-6 w-6 place-items-center rounded text-neutral-500 hover:bg-black/[0.06] disabled:opacity-25 dark:text-[#8B92A1] dark:hover:bg-white/[0.08]"
              >
                ↑
              </button>
              <button
                type="button"
                aria-label={t("ruleDown", covers(rule))}
                disabled={i === rules.length - 1}
                onClick={() => move(i, 1)}
                className="grid h-6 w-6 place-items-center rounded text-neutral-500 hover:bg-black/[0.06] disabled:opacity-25 dark:text-[#8B92A1] dark:hover:bg-white/[0.08]"
              >
                ↓
              </button>
              <button
                type="button"
                aria-label={t("ruleRemove", covers(rule))}
                onClick={() => load("rules_remove", { id: rule.id })}
                className="grid h-6 w-6 place-items-center rounded text-neutral-500 hover:bg-[#C0362F]/10 hover:text-[#C0362F] dark:text-[#8B92A1] dark:hover:bg-[#FF8A85]/10 dark:hover:text-[#FF8A85]"
              >
                ✕
              </button>
            </span>
          </li>
        ))}
      </ul>

      <p className="text-[11px] text-neutral-400 dark:text-[#646B7C]">{t("rulesFooter")}</p>
    </section>
  );
}
