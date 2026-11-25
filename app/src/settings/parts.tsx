import type { ReactNode } from "react";

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
    <div className="overflow-hidden rounded-lg border border-black/[0.08] bg-black/[0.015] dark:border-white/[0.08] dark:bg-white/[0.02]">
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
    <div className="flex items-center gap-3 border-black/[0.08] px-3.5 py-3 not-first:border-t dark:border-white/[0.08]">
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
      className={`relative h-[19px] w-[34px] shrink-0 rounded-full transition disabled:opacity-40 ${
        on ? "bg-accent" : "bg-black/[0.12] dark:bg-white/[0.12]"
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
