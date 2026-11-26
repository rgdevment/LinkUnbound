import { invoke } from "@tauri-apps/api/core";
import { type KeyboardEvent, type RefObject, useEffect, useId, useRef, useState } from "react";
import { useWords } from "../../i18n";
import Glyph, { AppIcon } from "./Glyph";
import { marked } from "./RuleRow";

export type SeenApp = {
  name: string;
  links: number;
  last_seen: number | null;
  icon: string | null;
  rule_id: string | null;
};

type Words = ReturnType<typeof useWords>;
type Choice = { app: SeenApp } | { use: string };

const DAY = 86_400_000;

function seenApps(): Promise<SeenApp[]> {
  return invoke<SeenApp[]>("apps_seen").catch(() => []);
}

/// By the calendar, not by hours elapsed: a link at 23:50 was yesterday ten minutes later.
export function seenSince(t: Words, seconds: number, now: Date): string {
  const midnight = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
  const days = Math.round((midnight(now) - midnight(new Date(seconds * 1000))) / DAY);
  if (days <= 0) return t("appSeenToday");
  if (days === 1) return t("appSeenYesterday");
  if (days < 14) return t("appSeenDays", String(days));
  return t("appSeenWeeks", String(Math.floor(days / 7)));
}

function about(t: Words, app: SeenApp, held: boolean, now: Date): string {
  if (held) return t("appHasRule");
  if (app.last_seen === null) return t("appNamedInRule");
  const links = app.links === 1 ? t("appLinksOne") : t("appLinksMany", String(app.links));
  return `${links} · ${seenSince(t, app.last_seen, now)}`;
}

const sameName = (a: string, b: string) =>
  a.trim().toLocaleLowerCase() === b.trim().toLocaleLowerCase();

/// The apps the picker saw links come from, most recent first, and room to name one it has not.
export default function AppPicker({
  label,
  value,
  onChange,
  editingId,
  startOpen,
  inputRef,
}: Readonly<{
  label: string;
  value: string;
  onChange: (name: string) => void;
  editingId?: string;
  startOpen: boolean;
  inputRef: RefObject<HTMLInputElement | null>;
}>) {
  const t = useWords();
  const id = useId();
  const [apps, setApps] = useState<SeenApp[] | null>(null);
  const [open, setOpen] = useState(startOpen);
  const [typed, setTyped] = useState(false);
  const [lit, setLit] = useState<number | null>(null);
  const box = useRef<HTMLDivElement>(null);

  useEffect(() => {
    void seenApps().then(setApps);
  }, []);

  const close = () => {
    setOpen(false);
    setTyped(false);
    setLit(null);
  };

  useEffect(() => {
    if (!open) return;
    const away = (e: MouseEvent) => {
      if (!box.current?.contains(e.target as Node)) {
        setOpen(false);
        setTyped(false);
        setLit(null);
      }
    };
    document.addEventListener("mousedown", away);
    return () => document.removeEventListener("mousedown", away);
  }, [open]);

  // Asked again on every opening: «open a link and come back» has to find it there.
  const show = () => {
    setOpen(true);
    void seenApps().then(setApps);
  };

  const all = apps ?? [];
  const query = typed ? value.trim() : "";
  const hits = all.filter((a) => a.name.toLocaleLowerCase().includes(query.toLocaleLowerCase()));
  const choices: Choice[] = hits.map((app) => ({ app }));
  if (query && !all.some((a) => sameName(a.name, query))) choices.push({ use: query });
  const held = (app: SeenApp) => app.rule_id !== null && app.rule_id !== editingId;
  const usable = (c: Choice) => !("app" in c) || !held(c.app);
  const chosen = choices.findIndex((c) => "app" in c && sameName(c.app.name, value));
  const preferred = !typed && chosen >= 0 && usable(choices[chosen]) ? chosen : -1;
  const active = lit ?? (preferred >= 0 ? preferred : choices.findIndex(usable));
  const face = all.find((a) => sameName(a.name, value));
  const now = new Date();

  const pick = (at: number) => {
    const choice = choices[at];
    if (choice && usable(choice)) onChange("app" in choice ? choice.app.name : choice.use);
    close();
  };

  const step = (by: number) => {
    for (let at = active + by; at >= 0 && at < choices.length; at += by) {
      if (usable(choices[at])) return at;
    }
    return active;
  };

  const keys = (e: KeyboardEvent) => {
    const by = { ArrowDown: 1, ArrowUp: -1 }[e.key];
    if (!open) {
      if (by) {
        e.preventDefault();
        show();
      }
      return;
    }
    if (by) {
      e.preventDefault();
      setLit(step(by));
    } else if (e.key === "Enter") {
      e.preventDefault();
      pick(active);
    } else if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      close();
    } else if (e.key === "Tab") {
      close();
    }
  };

  const useAt = choices.findIndex((c) => "use" in c);

  return (
    <div ref={box} className="relative min-w-0">
      <div className="flex items-center gap-2 rounded-md border border-(--hair) bg-panel px-2.5 py-1.5 text-sm has-focus-visible:outline-2 has-focus-visible:outline-offset-1 has-focus-visible:outline-accent">
        {face && <AppIcon src={face.icon} />}
        <input
          ref={inputRef}
          role="combobox"
          aria-label={label}
          aria-expanded={open}
          aria-controls={`${id}-list`}
          aria-autocomplete="list"
          aria-activedescendant={open && active >= 0 ? `${id}-${active}` : undefined}
          placeholder={label}
          value={value}
          onChange={(e) => {
            onChange(e.target.value);
            setTyped(true);
            setLit(null);
            if (!open) show();
          }}
          onClick={() => !open && show()}
          onKeyDown={keys}
          className="min-w-0 flex-1 bg-transparent outline-none placeholder:text-muted"
        />
        <button
          type="button"
          tabIndex={-1}
          aria-hidden="true"
          onClick={() => {
            if (open) close();
            else show();
            inputRef.current?.focus();
          }}
          className="text-muted"
        >
          <Glyph name="chev" size={12} />
        </button>
      </div>
      {open && apps !== null && (
        <div
          id={`${id}-list`}
          role="listbox"
          aria-label={label}
          tabIndex={-1}
          onKeyDown={keys}
          onMouseDown={(e) => e.preventDefault()}
          onClick={(e) => {
            const option = (e.target as HTMLElement).closest<HTMLElement>("[data-at]");
            if (option) pick(Number(option.dataset.at));
          }}
          className="absolute top-full right-0 left-0 z-20 mt-1 max-h-[300px] overflow-y-auto rounded-lg border border-(--hair) bg-panel p-1 text-sm shadow-lg outline-none"
        >
          {all.length === 0 ? (
            <div className="flex gap-2.5 px-2 pt-2.5 pb-2">
              <span className="grid size-8 shrink-0 place-items-center rounded-lg bg-(--tile) text-muted">
                <Glyph name="window" size={16} />
              </span>
              <div>
                <p className="text-sm font-semibold">{t("appsNoneTitle")}</p>
                <p className="mt-0.5 max-w-[56ch] text-xs text-muted">{t("appsNoneBody")}</p>
              </div>
            </div>
          ) : (
            <p className="px-2 pt-1.5 pb-0.5 text-xs text-muted">{t("appsSeenTitle")}</p>
          )}
          {all.length > 0 && hits.length === 0 && (
            <p className="px-2 pt-1 pb-1.5 text-xs text-muted">{t("appsNoMatch")}</p>
          )}
          {hits.map((app, at) => {
            const off = held(app);
            return (
              <div
                key={app.name}
                data-at={at}
                id={`${id}-${at}`}
                role="option"
                aria-selected={at === chosen}
                aria-disabled={off || undefined}
                tabIndex={-1}
                onMouseEnter={() => !off && setLit(at)}
                className={`flex cursor-default items-center gap-2 rounded-md px-2 py-1.5 whitespace-nowrap ${at === active ? "bg-(--accent-soft)" : ""} ${off ? "opacity-50" : ""}`}
              >
                <AppIcon src={app.icon} />
                <span className="min-w-0 truncate">{marked(app.name, query)}</span>
                <span className="ml-auto pl-3 text-xs text-muted tabular-nums">
                  {about(t, app, off, now)}
                </span>
                {at === chosen && (
                  <span className="text-accent">
                    <Glyph name="check" />
                  </span>
                )}
              </div>
            );
          })}
          {useAt >= 0 && (
            <>
              <div className="mx-0.5 my-1 h-px bg-(--hair)" />
              <div
                data-at={useAt}
                id={`${id}-${useAt}`}
                role="option"
                aria-selected={false}
                tabIndex={-1}
                onMouseEnter={() => setLit(useAt)}
                className={`flex cursor-default items-start gap-2 rounded-md px-2 py-1.5 ${useAt === active ? "bg-(--accent-soft)" : ""}`}
              >
                <span className="grid size-4 shrink-0 place-items-center text-muted">
                  <Glyph name="pencil" size={13} />
                </span>
                <span className="flex min-w-0 flex-col gap-px">
                  <span>{t("appUse", query)}</span>
                  <span className="text-xs text-muted">{t("appUseNote", query)}</span>
                </span>
              </div>
            </>
          )}
        </div>
      )}
    </div>
  );
}
