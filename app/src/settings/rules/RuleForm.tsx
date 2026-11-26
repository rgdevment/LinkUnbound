import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { type Key, useSpoken, useWords } from "../../i18n";
import { saidPlainly } from "../../refusal";
import DestinationPicker from "./DestinationPicker";
import Glyph, { BrowserIcon } from "./Glyph";
import { covers, type Destination, named, type RuleView, unnamed } from "./types";

export type FormKind = "site" | "host" | "url" | "app";

const KINDS: { id: FormKind; label: Key; hint: Key }[] = [
  { id: "site", label: "ruleNewKindSite", hint: "ruleNewValueSite" },
  { id: "host", label: "ruleNewKindHost", hint: "ruleNewValueHost" },
  { id: "url", label: "ruleNewKindUrl", hint: "ruleNewValueUrl" },
  { id: "app", label: "ruleNewKindApp", hint: "ruleNewValueApp" },
];

export type Draft = { kind: FormKind; value: string };

export function draftOf(rule: RuleView): Draft {
  if (rule.source_app && rule.kind === "any") return { kind: "app", value: rule.source_app };
  return { kind: rule.kind === "any" ? "site" : rule.kind, value: rule.covers };
}

export default function RuleForm({
  choices,
  start,
  editing,
  onSaved,
  onCancel,
}: Readonly<{
  choices: Destination[];
  start?: Draft;
  editing?: RuleView;
  onSaved: (rules: RuleView[]) => void;
  onCancel: () => void;
}>) {
  const t = useWords();
  const language = useSpoken();
  const [kind, setKind] = useState<FormKind>(start?.kind ?? "site");
  const [value, setValue] = useState(start?.value ?? "");
  const [where, setWhere] = useState(() => {
    if (editing) return named(editing.browser_id, editing.profile_id);
    return choices[0] ? named(choices[0].browserId, choices[0].profileId) : "";
  });
  const [isPrivate, setPrivate] = useState(editing?.private ?? false);
  const [saving, setSaving] = useState(false);
  const [refusal, setRefusal] = useState<string | null>(null);
  const hint = KINDS.find((k) => k.id === kind)?.hint ?? "ruleNewValueSite";
  const destination = choices.find((c) => named(c.browserId, c.profileId) === where);
  // A browser with no private window cannot be asked for one: the box is greyed rather than the
  // rule refused after the fact.
  const opensPrivately = destination?.opensPrivately ?? true;

  // The catalogue is read off the registry and may land after the form opened.
  useEffect(() => {
    if (where === "" && choices[0]) setWhere(named(choices[0].browserId, choices[0].profileId));
  }, [choices, where]);

  const valueBox = useRef<HTMLInputElement>(null);
  useEffect(() => valueBox.current?.focus(), []);

  const field =
    "rounded-md border border-(--hair) bg-panel px-2.5 py-1.5 text-sm focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent";

  return (
    <form
      aria-label={editing ? t("ruleEdit", covers(editing)) : t("ruleNewTitle")}
      className={
        editing
          ? "flex flex-col gap-2.5 rounded-[inherit] bg-panel p-3 shadow-[inset_0_0_0_1.5px_var(--accent)]"
          : "flex flex-col gap-2.5 rounded-lg border border-(--hair) bg-card p-3.5"
      }
      onSubmit={(e) => {
        e.preventDefault();
        const { browserId, profileId } = unnamed(where);
        setSaving(true);
        setRefusal(null);
        const asked = {
          kind,
          value,
          browserId,
          profileId,
          private: isPrivate && opensPrivately,
        };
        void invoke<RuleView[]>(
          editing ? "rules_edit" : "rules_add",
          editing ? { id: editing.id, ...asked } : asked,
        )
          .then(onSaved)
          .catch((err: unknown) => setRefusal(saidPlainly(language, err)))
          .finally(() => setSaving(false));
      }}
    >
      {!editing && <p className="text-md font-semibold">{t("ruleNewTitle")}</p>}
      <fieldset className="m-0 flex min-w-0 flex-wrap items-center gap-2 border-0 p-0">
        <legend className="float-left mr-2 text-sm text-muted">{t("ruleNewKind")}</legend>
        <div className="flex flex-wrap gap-0.5 rounded-md bg-(--tile) p-0.5">
          {KINDS.map((k) => (
            <label
              key={k.id}
              className={`rounded px-2 py-1 text-sm whitespace-nowrap has-focus-visible:outline-2 has-focus-visible:outline-accent ${
                kind === k.id ? "bg-panel font-semibold shadow-sm" : "text-ink-2"
              }`}
            >
              <input
                type="radio"
                name={`rule-kind-${editing?.id ?? "new"}`}
                value={k.id}
                checked={kind === k.id}
                onChange={() => setKind(k.id)}
                className="sr-only"
              />
              {t(k.label)}
            </label>
          ))}
        </div>
      </fieldset>
      <input
        aria-label={t(hint)}
        value={value}
        placeholder={t(hint)}
        onChange={(e) => setValue(e.target.value)}
        ref={valueBox}
        className={`${field} min-w-0`}
      />
      <div className="flex flex-wrap items-center gap-x-3 gap-y-2">
        <span className="text-sm text-muted">{t("ruleNewBrowser")}</span>
        <DestinationPicker
          label={t("ruleNewBrowser")}
          choices={choices}
          value={where}
          onChange={setWhere}
          face={
            destination ? (
              <>
                <BrowserIcon src={destination.icon} />
                <span className="truncate">{destination.label}</span>
              </>
            ) : (
              <>
                <span className="text-caution">
                  <Glyph name="warn" />
                </span>
                <span className="truncate">{editing?.browser ?? ""}</span>
              </>
            )
          }
        />
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={isPrivate && opensPrivately}
            disabled={!opensPrivately}
            onChange={(e) => setPrivate(e.target.checked)}
          />
          {t("ruleNewPrivate")}
        </label>
      </div>
      {refusal && (
        <p role="alert" className="rounded-md bg-(--danger-soft) px-3 py-2 text-sm text-danger">
          {refusal}
        </p>
      )}
      {choices.length === 0 && <p className="text-sm text-muted">{t("browsersNone")}</p>}
      <div className="flex flex-wrap items-center gap-2">
        <button
          type="submit"
          disabled={saving || choices.length === 0}
          className="rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-on-accent disabled:opacity-50"
        >
          {t("save")}
        </button>
        <button
          type="button"
          onClick={onCancel}
          className="rounded-md border border-(--hair) px-3 py-1.5 text-sm"
        >
          {t("cancel")}
        </button>
        {editing && <p className="ml-auto text-xs text-muted">{t("ruleEditKeeps")}</p>}
      </div>
    </form>
  );
}
