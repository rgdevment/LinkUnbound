import { type KeyboardEvent, type ReactNode, useEffect, useId, useRef, useState } from "react";
import { useWords } from "../../i18n";
import Glyph, { BrowserIcon } from "./Glyph";
import { type Destination, named } from "./types";

/// A select can carry no icon, and the icon is how people tell their browsers apart.
export default function DestinationPicker({
  label,
  choices,
  value,
  face,
  onChange,
  needsPrivate = false,
  inline = false,
  open: asked,
  onOpenChange,
}: {
  label: string;
  choices: Destination[];
  value: string;
  face: ReactNode;
  onChange: (key: string) => void;
  needsPrivate?: boolean;
  inline?: boolean;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}) {
  const t = useWords();
  const id = useId();
  const [selfOpen, setSelfOpen] = useState(false);
  const open = asked ?? selfOpen;
  const setOpen = onOpenChange ?? setSelfOpen;
  const usable = (c: Destination) => !needsPrivate || c.opensPrivately;
  const chosen = choices.findIndex((c) => named(c.browserId, c.profileId) === value);
  const [hovered, setHovered] = useState<number | null>(null);
  const active = hovered ?? (chosen >= 0 ? chosen : Math.max(0, choices.findIndex(usable)));
  const button = useRef<HTMLButtonElement>(null);
  const list = useRef<HTMLDivElement>(null);
  const box = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (open) list.current?.focus();
    else setHovered(null);
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const away = (e: MouseEvent) => {
      if (!box.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", away);
    return () => document.removeEventListener("mousedown", away);
  }, [open, setOpen]);

  const pick = (at: number) => {
    const choice = choices[at];
    if (!choice || !usable(choice)) return;
    setOpen(false);
    button.current?.focus();
    onChange(named(choice.browserId, choice.profileId));
  };

  const step = (from: number, by: number) => {
    for (let at = from + by; at >= 0 && at < choices.length; at += by) {
      if (usable(choices[at])) return at;
    }
    return from;
  };

  const keys = (e: KeyboardEvent) => {
    const moved =
      e.key === "ArrowDown"
        ? step(active, 1)
        : e.key === "ArrowUp"
          ? step(active, -1)
          : e.key === "Home"
            ? step(-1, 1)
            : e.key === "End"
              ? step(choices.length, -1)
              : null;
    if (moved !== null) {
      e.preventDefault();
      setHovered(moved);
    } else if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      pick(active);
    } else if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      setOpen(false);
      button.current?.focus();
    } else if (e.key === "Tab") {
      setOpen(false);
    }
  };

  return (
    <div ref={box} className="relative min-w-0">
      <button
        type="button"
        ref={button}
        aria-label={label}
        aria-haspopup="listbox"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
        className={
          inline
            ? "flex max-w-full items-center gap-1.5 rounded-md border border-transparent px-1.5 py-0.5 text-[12px] group-hover:border-(--hair) hover:border-(--hair) focus-visible:outline-2 focus-visible:outline-(--accent)"
            : "flex min-w-[200px] items-center gap-2 rounded-md border border-(--hair) bg-(--panel) px-2.5 py-1.5 text-[12px] focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-(--accent)"
        }
      >
        {face}
        <span className="ml-auto text-(--muted)">
          <Glyph name="chev" size={12} />
        </span>
      </button>
      {open && (
        <div
          ref={list}
          role="listbox"
          aria-label={label}
          tabIndex={-1}
          aria-activedescendant={`${id}-${active}`}
          onKeyDown={keys}
          className={`absolute top-full z-20 ${inline ? "right-0" : "left-0"} mt-1 max-h-[260px] min-w-[260px] overflow-y-auto rounded-lg border border-(--hair) bg-(--panel) p-1 text-[12px] shadow-lg outline-none`}
        >
          {choices.map((choice, at) => {
            const key = named(choice.browserId, choice.profileId);
            const off = !usable(choice);
            return (
              // biome-ignore lint/a11y/useKeyWithClickEvents: the listbox takes the keys for every option
              <div
                key={key}
                id={`${id}-${at}`}
                role="option"
                aria-selected={key === value}
                aria-disabled={off || undefined}
                tabIndex={-1}
                onClick={() => pick(at)}
                onMouseEnter={() => !off && setHovered(at)}
                className={`flex cursor-default items-center gap-2 rounded-md px-2 py-1.5 whitespace-nowrap ${
                  at === active ? "bg-(--accent-soft)" : ""
                } ${off ? "opacity-50" : ""}`}
              >
                <BrowserIcon src={choice.icon} />
                <span>{choice.label}</span>
                {!choice.opensPrivately && (
                  <span className="ml-auto pl-3 text-[11px] text-(--muted)">
                    {t("ruleNoPrivate")}
                  </span>
                )}
                {key === value && (
                  <span
                    className={`${choice.opensPrivately ? "ml-auto pl-3" : ""} text-(--accent)`}
                  >
                    <Glyph name="check" />
                  </span>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
