import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { type Key, useSpoken, useWords } from "../i18n";
import { saidPlainly } from "../refusal";
import Confirm from "./Confirm";
import Glyph from "./rules/Glyph";
import RuleForm, { type Draft, draftOf } from "./rules/RuleForm";
import RuleProbe from "./rules/RuleProbe";
import RuleRow, { KindChip } from "./rules/RuleRow";
import {
  type BrowserView,
  covers,
  type Destination,
  destinationsOf,
  type Family,
  familyOf,
  type RuleView,
  unnamed,
} from "./rules/types";

type Filter = "all" | Family | "broken";

const FILTERS: { id: Filter; label: Key }[] = [
  { id: "all", label: "rulesAll" },
  { id: "app", label: "kindFrom" },
  { id: "url", label: "kindUrl" },
  { id: "host", label: "kindHost" },
  { id: "site", label: "kindSite" },
];

/// The search looks at everything a row shows: what it covers, the app, the browser, the profile.
function found(rules: RuleView[], query: string): RuleView[] {
  const q = query.trim().toLocaleLowerCase();
  if (!q) return rules;
  return rules.filter((r) =>
    [covers(r), r.source_app, r.browser, r.profile].some((s) => s?.toLocaleLowerCase().includes(q)),
  );
}

function kept(rule: RuleView, filter: Filter): boolean {
  if (filter === "all") return true;
  if (filter === "broken") return !rule.resolved;
  return familyOf(rule) === filter;
}

function RankLine() {
  const t = useWords();
  const order: Family[] = ["app", "url", "host", "site"];
  return (
    <p className="flex flex-wrap items-center gap-1.5 text-sm text-muted">
      <Glyph name="info" />
      <span>{t("rulesRank")}</span>
      {order.map((family, at) => (
        <span key={family} className="flex items-center gap-1.5">
          {at > 0 && <span aria-hidden="true">›</span>}
          <KindChip family={family} />
        </span>
      ))}
      <span>{t("rulesRankTie")}</span>
    </p>
  );
}

export default function Rules() {
  const t = useWords();
  const language = useSpoken();
  const [rules, setRules] = useState<RuleView[] | null>(null);
  const [choices, setChoices] = useState<Destination[]>([]);
  const [problem, setProblem] = useState<string | null>(null);
  const [refusal, setRefusal] = useState<string | null>(null);
  const [asking, setAsking] = useState<RuleView | null>(null);
  const [adding, setAdding] = useState<Draft | null>(null);
  const [editing, setEditing] = useState<string | null>(null);
  const [picking, setPicking] = useState<string | null>(null);
  const [probing, setProbing] = useState(false);
  const [winner, setWinner] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<Filter>("all");

  useEffect(() => {
    void invoke<RuleView[]>("rules_list")
      .then(setRules)
      .catch((e: unknown) => setProblem(saidPlainly(language, e)));
  }, [language]);

  useEffect(() => {
    void invoke<BrowserView[]>("browsers_list")
      .then((all) => setChoices(destinationsOf(all)))
      .catch(() => setChoices([]));
  }, []);

  // A refused change keeps the list on screen and says why above it.
  const act = (command: string, args: Record<string, unknown>) => {
    void invoke<RuleView[]>(command, args)
      .then((next) => {
        setRules(next);
        setRefusal(null);
      })
      .catch((e: unknown) => setRefusal(saidPlainly(language, e)));
  };

  if (problem) {
    return <p className="rounded-md bg-(--danger-soft) px-3 py-2 text-sm text-danger">{problem}</p>;
  }

  if (rules === null) return null;

  const addButton = (
    <button
      type="button"
      onClick={() => setAdding({ kind: "site", value: "" })}
      className="flex items-center gap-1.5 rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-on-accent"
    >
      <Glyph name="plus" />
      {t("ruleAdd")}
    </button>
  );
  const form = adding && (
    <RuleForm
      key={`${adding.kind}:${adding.value}`}
      choices={choices}
      start={adding}
      onSaved={(next) => {
        setRules(next);
        setAdding(null);
        setRefusal(null);
      }}
      onCancel={() => setAdding(null)}
    />
  );

  if (rules.length === 0) {
    return (
      <div className="flex flex-col gap-3">
        {form}
        {!adding && (
          <div className="flex flex-col items-center gap-3 rounded-lg border border-dashed border-(--hair) px-5 py-7 text-center">
            <span className="grid h-10 w-10 place-items-center rounded-[10px] bg-(--accent-soft) text-accent">
              <Glyph name="rules" size={20} />
            </span>
            <div>
              <p className="text-md font-semibold">{t("rulesEmptyTitle")}</p>
              <p className="mt-1 max-w-[46ch] text-sm text-muted">{t("rulesEmptyBody")}</p>
            </div>
            {addButton}
          </div>
        )}
      </div>
    );
  }

  const broken = rules.some((r) => !r.resolved);
  // Fixing the last broken rule takes its chip away; the filter must not stay on it.
  const showing = filter === "broken" && !broken ? "all" : filter;
  const searched = found(rules, query);
  const shown = searched.filter((r) => kept(r, showing));
  const count = (f: Filter) => searched.filter((r) => kept(r, f)).length;
  let title = rules.length === 1 ? t("rulesOne") : t("rulesMany", String(rules.length));
  if (shown.length !== rules.length) {
    title = t("rulesSome", String(shown.length), String(rules.length));
  }
  const apps = [...new Set(rules.flatMap((r) => (r.source_app ? [r.source_app] : [])))];

  return (
    <section className="flex flex-col gap-3">
      {asking && (
        <Confirm
          title={t("ruleDeleteTitle", covers(asking))}
          body={t("ruleDeleteBody")}
          go={t("deleteGo")}
          onConfirm={() => {
            act("rules_remove", { id: asking.id });
            setAsking(null);
          }}
          onCancel={() => setAsking(null)}
        />
      )}
      <div className="flex flex-wrap items-center gap-2">
        <h2 className="flex-1 text-xs font-semibold tracking-[0.09em] text-muted uppercase">
          {title}
        </h2>
        <button
          type="button"
          aria-pressed={probing}
          onClick={() => {
            setProbing(!probing);
            setWinner(null);
          }}
          className="flex items-center gap-1.5 rounded-md border border-(--hair) px-3 py-1.5 text-sm font-medium"
        >
          <Glyph name="flask" />
          {t("probeOpen")}
        </button>
        {!adding && addButton}
      </div>

      {probing && (
        <RuleProbe
          rules={rules}
          apps={apps}
          onWinner={setWinner}
          onCreate={(site) => setAdding({ kind: "site", value: site })}
          onClose={() => {
            setProbing(false);
            setWinner(null);
          }}
        />
      )}
      {form}
      {refusal && (
        <p role="alert" className="rounded-md bg-(--danger-soft) px-3 py-2 text-sm text-danger">
          {refusal}
        </p>
      )}

      <div className="flex flex-col gap-2">
        <label className="flex min-w-0 items-center gap-2 rounded-md border border-(--hair) bg-panel px-2.5 py-1.5 text-sm focus-within:outline-2 focus-within:outline-offset-1 focus-within:outline-accent">
          <span className="text-muted">
            <Glyph name="search" />
          </span>
          <input
            type="search"
            aria-label={t("rulesSearch")}
            placeholder={t("rulesSearch")}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="min-w-0 flex-1 bg-transparent outline-none placeholder:text-muted [&::-webkit-search-cancel-button]:appearance-none"
          />
          {query && (
            <button
              type="button"
              aria-label={t("rulesSearchClear")}
              onClick={() => setQuery("")}
              className="text-muted hover:text-ink"
            >
              <Glyph name="x" size={12} />
            </button>
          )}
        </label>
        <fieldset
          aria-label={t("rulesFilter")}
          className="m-0 flex min-w-0 flex-wrap gap-1 border-0 p-0"
        >
          {FILTERS.map((f) => (
            <button
              key={f.id}
              type="button"
              aria-pressed={showing === f.id}
              onClick={() => setFilter(f.id)}
              className={`rounded-md px-[9px] py-[3px] text-sm whitespace-nowrap ${
                showing === f.id
                  ? "bg-(--accent-soft) font-semibold text-accent"
                  : "bg-(--tile) text-ink-2"
              }`}
            >
              {t(f.label)}
              <span className="ml-1 text-muted tabular-nums">{count(f.id)}</span>
            </button>
          ))}
          {broken && (
            <button
              type="button"
              aria-pressed={showing === "broken"}
              onClick={() => setFilter("broken")}
              className={`flex items-center gap-1 rounded-md px-[9px] py-[3px] text-sm whitespace-nowrap text-caution ${
                showing === "broken" ? "bg-caution/[0.12] font-semibold" : "bg-(--tile)"
              }`}
            >
              <Glyph name="warn" size={12} />
              {t("rulesBroken")}
              <span className="ml-1 tabular-nums">{count("broken")}</span>
            </button>
          )}
        </fieldset>
      </div>
      <RankLine />

      {shown.length === 0 ? (
        <p className="rounded-lg border border-dashed border-(--hair) px-4 py-5 text-center text-sm text-muted">
          {t("rulesNoneFound")}
        </p>
      ) : (
        <ul className="rounded-lg border border-(--hair)">
          {shown.map((rule) =>
            rule.id === editing ? (
              <li
                key={rule.id}
                className="border-(--hair) not-first:border-t first:rounded-t-lg last:rounded-b-lg"
              >
                <RuleForm
                  choices={choices}
                  start={draftOf(rule)}
                  editing={rule}
                  onSaved={(next) => {
                    setRules(next);
                    setEditing(null);
                    setRefusal(null);
                  }}
                  onCancel={() => setEditing(null)}
                />
              </li>
            ) : (
              <RuleRow
                key={rule.id}
                rule={rule}
                query={query}
                verdict={rule.id === winner ? "wins" : null}
                choices={choices}
                picking={picking === rule.id}
                onPicking={(open) => setPicking(open ? rule.id : null)}
                onRetarget={(key) => act("rules_retarget", { id: rule.id, ...unnamed(key) })}
                onEdit={() => setEditing(rule.id)}
                onRemove={() => setAsking(rule)}
              />
            ),
          )}
        </ul>
      )}

      <p className="text-xs text-muted">{t("rulesFooter")}</p>
    </section>
  );
}
