import { useEffect, useRef } from "react";
import { useWords } from "../i18n";

/// Shared so the three destructive spots agree on the focus handling a screen
/// reader needs: take it on mount, release it back where it came from.
export default function Confirm({
  title,
  body,
  go,
  onConfirm,
  onCancel,
}: {
  title: string;
  body: string;
  go: string;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  const t = useWords();
  const first = useRef<HTMLButtonElement>(null);
  const came = useRef<Element | null>(null);

  useEffect(() => {
    came.current = document.activeElement;
    first.current?.focus();
    return () => {
      if (came.current instanceof HTMLElement) came.current.focus();
    };
  }, []);

  return (
    <div
      role="alertdialog"
      aria-modal="true"
      aria-label={title}
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.stopPropagation();
          onCancel();
        }
      }}
      className="rounded-lg border border-black/[0.12] bg-black/[0.02] p-3.5 dark:border-white/[0.12] dark:bg-white/[0.03]"
    >
      <p className="text-md font-semibold">{title}</p>
      <p className="mt-1 text-sm text-muted">{body}</p>
      <div className="mt-3 flex gap-2">
        <button
          type="button"
          ref={first}
          onClick={onConfirm}
          className="rounded-md bg-danger px-3 py-1.5 text-sm font-medium text-on-accent"
        >
          {go}
        </button>
        <button
          type="button"
          onClick={onCancel}
          className="rounded-md px-3 py-1.5 text-sm text-muted"
        >
          {t("cancel")}
        </button>
      </div>
    </div>
  );
}
