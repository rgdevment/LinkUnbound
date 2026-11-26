import type { ReactNode } from "react";
import { type Key, useWords } from "../i18n";

export function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="flex flex-col gap-2">
      <h2 className="text-xs font-semibold tracking-[0.09em] text-muted uppercase">{title}</h2>
      {children}
    </section>
  );
}

export function Card({ children }: { children: ReactNode }) {
  return (
    <div className="overflow-hidden rounded-lg border border-hair bg-black/[0.015] dark:bg-white/[0.02]">
      {children}
    </div>
  );
}

export function Line({
  title,
  note,
  children,
}: {
  title: string;
  note?: string;
  children?: ReactNode;
}) {
  return (
    <div className="flex items-center gap-3 border-hair px-3.5 py-3 not-first:border-t">
      <div className="min-w-0 flex-1">
        <p className="text-md">{title}</p>
        {note && <p className="mt-px text-sm text-muted">{note}</p>}
      </div>
      {children}
    </div>
  );
}

export function Switch({
  on,
  disabled,
  label,
  onChange,
}: {
  on: boolean;
  disabled?: boolean;
  label: string;
  onChange: (next: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      aria-label={label}
      disabled={disabled}
      onClick={() => onChange(!on)}
      className={`relative h-[19px] w-[34px] shrink-0 rounded-full transition focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent enabled:cursor-pointer enabled:hover:ring-4 enabled:hover:ring-accent/20 disabled:opacity-40 ${
        on ? "bg-accent" : "bg-edge enabled:hover:bg-ink-2/40"
      }`}
    >
      <span
        className={`absolute top-[3px] block h-[13px] w-[13px] rounded-full transition-all ${
          on ? "left-[18px] bg-white" : "left-[3px] bg-muted"
        }`}
      />
    </button>
  );
}

const CHOSEN_HOVER = "cursor-pointer hover:bg-accent/[0.2] dark:hover:bg-accent/[0.26]";
const OTHER_HOVER =
  "cursor-pointer hover:bg-black/[0.09] hover:text-ink dark:hover:bg-white/[0.12]";

export function Choice<T extends string>({
  options,
  value,
  label,
  disabled,
  onPick,
}: {
  options: { id: T; label: Key }[];
  value: T;
  label: string;
  disabled?: boolean;
  onPick: (next: T) => void;
}) {
  const t = useWords();
  return (
    // Relative, so the hidden radios scroll with the page instead of stretching the window.
    <fieldset disabled={disabled} className="relative flex shrink-0 gap-1 disabled:opacity-40">
      <legend className="sr-only">{label}</legend>
      {options.map((o) => (
        <label key={o.id} className="contents">
          <input
            type="radio"
            name={label}
            className="peer sr-only"
            checked={o.id === value}
            onChange={() => onPick(o.id)}
          />
          <span
            className={`rounded-md px-2.5 py-1 text-sm transition peer-focus-visible:outline-2 peer-focus-visible:outline-offset-1 peer-focus-visible:outline-accent ${
              o.id === value
                ? "bg-accent/[0.13] font-semibold text-accent dark:bg-accent/[0.17]"
                : "bg-black/[0.045] text-ink-2 dark:bg-white/[0.06]"
            } ${disabled ? "" : o.id === value ? CHOSEN_HOVER : OTHER_HOVER}`}
          >
            {t(o.label)}
          </span>
        </label>
      ))}
    </fieldset>
  );
}
