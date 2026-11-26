import { type CSSProperties, type KeyboardEvent, useEffect, useId, useRef, useState } from "react";
import { useWords } from "../i18n";
import { familyCss } from "../theme";
import type { Fonts } from "./look";

type Pickable = { name: string; label: string };

/// The search and the list's tallest: what the drop needs beside the button.
const DROP_HEIGHT = 280;
const GAP = 4;

/// What Settings draws with when nothing is chosen, so «System» shows in its own letters too.
const SYSTEM_LETTERS =
  '"Segoe UI Variable Text", "Segoe UI", -apple-system, BlinkMacSystemFont, system-ui';

/// Fixed to the window, because the card around the line clips whatever hangs out of it. Below
/// while it fits, else on the roomier side, cut to the room there.
function dropAt(button: HTMLElement): CSSProperties {
  const at = button.getBoundingClientRect();
  const right = window.innerWidth - at.right;
  const below = window.innerHeight - at.bottom - 2 * GAP;
  const above = at.top - 2 * GAP;
  const up = below < DROP_HEIGHT && above > below;
  const maxHeight = Math.min(DROP_HEIGHT, up ? above : below);
  return up
    ? { right, bottom: window.innerHeight - at.top + GAP, maxHeight }
    : { right, top: at.bottom + GAP, maxHeight };
}

const folded = (said: string) =>
  said
    .normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
    .toLowerCase();

export function matching<T extends Pickable>(all: T[], query: string): T[] {
  const asked = folded(query.trim());
  if (!asked) return all;
  return all.filter((one) => folded(one.label).includes(asked) || folded(one.name).includes(asked));
}

/// «System» first, then every family; each option is written in its own letters.
export default function FontPicker({
  label,
  fonts,
  chosen,
  onPick,
}: {
  label: string;
  fonts: Fonts | null;
  chosen: string | null;
  onPick: (font: string | null) => void;
}) {
  const t = useWords();
  const [open, setOpen] = useState<CSSProperties | null>(null);
  const [query, setQuery] = useState("");
  const [lit, setLit] = useState(0);
  const opener = useRef<HTMLButtonElement>(null);
  const drop = useRef<HTMLDivElement>(null);
  const list = useId();
  const all: Pickable[] = [
    { name: "", label: t("lookFontSystem", fonts?.system ?? "…") },
    ...(fonts?.families ?? []),
  ];
  const current = all.find((one) => one.name === (chosen ?? "")) ?? all[0];
  const shown = matching(all, query);
  const lettersOf = (one: Pickable) => ({
    fontFamily: one.name ? familyCss(one.name) : SYSTEM_LETTERS,
  });

  const close = (refocus: boolean) => {
    setOpen(null);
    setQuery("");
    if (refocus) opener.current?.focus();
  };

  const choose = (one: Pickable) => {
    close(true);
    // Against the saved name: a family uninstalled since shows as System, and System must clear it.
    if (one.name !== (chosen ?? "")) onPick(one.name || null);
  };

  const toggle = () => {
    if (open) {
      close(true);
      return;
    }
    if (!opener.current) return;
    setLit(Math.max(0, all.indexOf(current)));
    setOpen(dropAt(opener.current));
  };

  // A page scrolled or a window resized under a fixed drop would leave it away from its button.
  useEffect(() => {
    if (!open) return;
    const away = (event: Event) => {
      if (!(event.target instanceof Node && drop.current?.contains(event.target))) close(false);
    };
    window.addEventListener("scroll", away, true);
    window.addEventListener("resize", away);
    return () => {
      window.removeEventListener("scroll", away, true);
      window.removeEventListener("resize", away);
    };
  });

  const steer = (event: KeyboardEvent<HTMLInputElement>) => {
    const last = shown.length - 1;
    const moves: Record<string, number> = {
      ArrowDown: Math.min(lit + 1, last),
      ArrowUp: Math.max(lit - 1, 0),
      Home: 0,
      End: last,
    };
    if (event.key in moves) {
      event.preventDefault();
      setLit(moves[event.key]);
      document.getElementById(`${list}-${moves[event.key]}`)?.scrollIntoView({ block: "nearest" });
    } else if (event.key === "Enter" && shown[lit]) {
      event.preventDefault();
      choose(shown[lit]);
    } else if (event.key === "Escape") {
      event.preventDefault();
      close(true);
    }
  };

  return (
    <div className="shrink-0">
      <button
        ref={opener}
        type="button"
        aria-label={`${label}: ${current.label}`}
        aria-haspopup="listbox"
        aria-expanded={open !== null}
        disabled={!fonts}
        style={lettersOf(current)}
        onMouseDown={(event) => {
          if (open) event.preventDefault();
        }}
        onClick={toggle}
        className="flex max-w-[260px] min-w-[200px] items-center enabled:cursor-pointer disabled:opacity-40 justify-between gap-2 rounded-md border border-edge px-3 py-1.5 text-sm text-ink focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      >
        <span className="truncate">{current.label}</span>
        <span aria-hidden="true" className="text-muted">
          ▾
        </span>
      </button>
      {open && (
        <div
          ref={drop}
          style={open}
          className="fixed z-20 flex w-[300px] flex-col rounded-lg border border-hair bg-panel p-1.5 shadow-lg"
        >
          <input
            type="search"
            role="combobox"
            aria-label={t("lookFontSearch")}
            aria-expanded="true"
            aria-controls={list}
            aria-activedescendant={shown[lit] ? `${list}-${lit}` : undefined}
            placeholder={t("lookFontSearch")}
            value={query}
            autoFocus
            onChange={(event) => {
              setQuery(event.target.value);
              setLit(0);
            }}
            onKeyDown={steer}
            onBlur={() => close(false)}
            className="w-full shrink-0 rounded-md border border-accent bg-panel px-2.5 py-1.5 text-sm text-ink outline-none placeholder:text-muted"
          />
          <div
            id={list}
            role="listbox"
            aria-label={label}
            onMouseDown={(event) => event.preventDefault()}
            className="scroller mt-1 max-h-[220px] min-h-0"
          >
            {shown.map((one, at) => (
              <div
                key={one.name || "system"}
                id={`${list}-${at}`}
                role="option"
                tabIndex={-1}
                aria-selected={one.name === current.name}
                style={lettersOf(one)}
                onMouseEnter={() => setLit(at)}
                onClick={() => choose(one)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" || event.key === " ") choose(one);
                }}
                className={`flex cursor-pointer items-center gap-2 truncate rounded-md px-2 py-1.5 text-md ${
                  at === lit ? "bg-accent/[0.13] dark:bg-accent/[0.17]" : ""
                } ${one.name === current.name ? "font-semibold text-accent" : "text-ink"}`}
              >
                {one.label}
              </div>
            ))}
            {shown.length === 0 && <p className="m-2 text-sm text-muted">{t("lookFontNone")}</p>}
          </div>
        </div>
      )}
    </div>
  );
}
