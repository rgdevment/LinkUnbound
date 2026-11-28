import type { ReactNode } from "react";
import { useWords } from "../../i18n";
import DestinationPicker from "./DestinationPicker";
import Glyph, { BrowserIcon } from "./Glyph";
import {
  covers,
  type Destination,
  FAMILY_LABEL,
  type Family,
  familyOf,
  named,
  type RuleView,
} from "./types";

const CHIP: Record<Family, string> = {
  app: "bg-caution/[0.08] text-caution",
  url: "bg-(--accent-soft) text-accent",
  host: "bg-(--tile) text-ink-2",
  site: "bg-(--tile) text-ink-2",
  file_type: "bg-(--tile) text-ink-2",
};

export function KindChip({ family }: Readonly<{ family: Family }>) {
  const t = useWords();
  return (
    <span
      className={`w-[84px] shrink-0 rounded px-1.5 py-0.5 text-center text-xs whitespace-nowrap ${CHIP[family]}`}
    >
      {t(FAMILY_LABEL[family])}
    </span>
  );
}

export function marked(text: string, query: string): ReactNode {
  const q = query.trim().toLocaleLowerCase();
  if (!q) return text;
  const lower = text.toLocaleLowerCase();
  const parts: ReactNode[] = [];
  let from = 0;
  for (let at = lower.indexOf(q); at >= 0; at = lower.indexOf(q, at + q.length)) {
    parts.push(
      text.slice(from, at),
      <mark key={at} className="rounded-sm bg-(--accent-soft) text-accent">
        {text.slice(at, at + q.length)}
      </mark>,
    );
    from = at + q.length;
  }
  parts.push(text.slice(from));
  return parts;
}

export function Where({ rule }: Readonly<{ rule: RuleView }>) {
  return (
    <>
      {rule.resolved ? (
        <BrowserIcon src={rule.icon} />
      ) : (
        <span className="text-caution">
          <Glyph name="warn" />
        </span>
      )}
      <span className={`truncate ${rule.resolved ? "" : "text-caution"}`}>{rule.browser}</span>
      {rule.profile && (
        <span className="shrink-0 rounded bg-(--tile) px-1.5 py-px text-xs text-ink-2">
          {rule.profile}
        </span>
      )}
    </>
  );
}

export type Verdict = "wins" | "also" | null;

/// The actions stay in sight: hidden until hover, nobody finds them by keyboard or touch.
export default function RuleRow({
  rule,
  query = "",
  verdict = null,
  choices,
  picking = false,
  onPicking,
  onRetarget,
  onEdit,
  onRemove,
}: Readonly<{
  rule: RuleView;
  query?: string;
  verdict?: Verdict;
  choices?: Destination[];
  picking?: boolean;
  onPicking?: (open: boolean) => void;
  onRetarget?: (key: string) => void;
  onEdit?: () => void;
  onRemove?: () => void;
}>) {
  const t = useWords();
  const what = covers(rule);
  // A picker rule can bind an app and a site at once; without its name it reads like any site rule.
  const app = rule.source_app && rule.kind !== "any" ? rule.source_app : null;
  const listed = choices !== undefined;
  let background = rule.resolved ? "bg-card" : "bg-caution/[0.08]";
  if (verdict === "wins")
    background = "bg-(--accent-soft) shadow-[inset_0_0_0_1.5px_var(--accent)]";

  return (
    <li
      className={`group flex flex-wrap items-center gap-x-3 gap-y-1.5 border-(--hair) py-[7px] pr-2 pl-3 not-first:border-t first:rounded-t-lg last:rounded-b-lg ${background}`}
    >
      <KindChip family={familyOf(rule)} />
      <span title={app ? `${what} · ${app}` : what} className="min-w-0 flex-1 truncate text-md">
        {marked(what, query)}
        {app && <span className="text-muted"> · {marked(t("ruleFromApp", app), query)}</span>}
      </span>
      {verdict && (
        <span
          className={`shrink-0 rounded-full px-[7px] py-px text-xs ${
            verdict === "wins" ? "bg-accent font-semibold text-on-accent" : "bg-(--tile) text-ink-2"
          }`}
        >
          {t(verdict === "wins" ? "probeWins" : "probeAlso")}
        </span>
      )}
      <span aria-hidden="true" className="shrink-0 text-xs text-muted">
        →
      </span>
      {listed && onRetarget && choices.length > 0 ? (
        <DestinationPicker
          inline
          label={t("ruleBrowser", what)}
          choices={choices}
          value={named(rule.browser_id, rule.profile_id)}
          needsPrivate={rule.private}
          open={picking}
          onOpenChange={onPicking}
          onChange={onRetarget}
          face={<Where rule={rule} />}
        />
      ) : (
        <span className="flex min-w-0 shrink-0 items-center gap-1.5 px-1.5 text-sm">
          <Where rule={rule} />
        </span>
      )}
      {rule.private && (
        <span className="flex shrink-0 items-center gap-1 text-xs text-accent">
          <Glyph name="mask" size={13} />
          {t("rulePrivate")}
        </span>
      )}
      {onEdit && onRemove && (
        <span className="flex shrink-0 gap-0.5">
          <button
            type="button"
            aria-label={t("ruleEdit", what)}
            title={t("ruleEdit", what)}
            onClick={onEdit}
            className="grid h-[26px] w-[26px] place-items-center rounded-md text-muted hover:bg-(--tile) hover:text-ink focus-visible:outline-2 focus-visible:outline-accent"
          >
            <Glyph name="pencil" />
          </button>
          <button
            type="button"
            aria-label={t("ruleRemove", what)}
            title={t("ruleRemove", what)}
            onClick={onRemove}
            className="grid h-[26px] w-[26px] place-items-center rounded-md text-muted hover:bg-(--danger-soft) hover:text-danger focus-visible:outline-2 focus-visible:outline-accent"
          >
            <Glyph name="trash" />
          </button>
        </span>
      )}
      {listed && !rule.resolved && (
        <div className="flex basis-full flex-wrap items-center gap-x-2.5 gap-y-1 pb-0.5 pl-[96px] text-sm text-caution">
          <span className="min-w-0 flex-1">
            {t(rule.profile_gone ? "ruleProfileGoneNote" : "ruleBrokenNote", rule.browser)}
          </span>
          {onPicking && choices.length > 0 && (
            <button
              type="button"
              onClick={() => onPicking(true)}
              className="rounded-md bg-caution px-3 py-1.5 text-sm font-medium text-on-accent hover:bg-caution/85 focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent"
            >
              {t("ruleBrokenPick")}
            </button>
          )}
        </div>
      )}
    </li>
  );
}
