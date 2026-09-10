import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";

type Profile = { id: string; name: string };
type Browser = {
  id: string;
  name: string;
  exe: string;
  profiles: Profile[];
  private_flag: string | null;
  icon: string | null;
};
type Destinations = { browsers: Browser[]; is_default: boolean };
type Incoming = { url: string; source_app: string | null; host: string; site: string };

/// One row per destination: a browser without profiles is one row, a browser
/// with them is one row each, because that is the choice being made.
type Destination = { browser: Browser; profile: Profile | null };

type Remember = "once" | "url" | "host" | "site";

function rowsOf(browsers: Browser[]): Destination[] {
  return browsers.flatMap((browser): Destination[] =>
    browser.profiles.length === 0
      ? [{ browser, profile: null }]
      : browser.profiles.map((profile) => ({ browser, profile })),
  );
}

function split(url: string): { head: string; tail: string } {
  try {
    const u = new URL(url);
    return { head: u.host, tail: u.pathname === "/" ? u.search : u.pathname + u.search };
  } catch {
    return { head: url, tail: "" };
  }
}

const COPY = (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.9"
    strokeLinecap="round"
    className="h-4 w-4"
    aria-hidden="true"
  >
    <rect x="9" y="9" width="12" height="12" rx="2.5" />
    <path d="M5 15V5.5A2.5 2.5 0 0 1 7.5 3H15" />
  </svg>
);

const MASK = (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.9"
    strokeLinecap="round"
    className="h-4 w-4"
    aria-hidden="true"
  >
    <path d="M3 12h18" />
    <path d="M5 12c0-3.3 1.6-5.5 3.4-5.5 1.5 0 2.3 1 3.6 1s2.1-1 3.6-1C17.4 6.5 19 8.7 19 12" />
    <ellipse cx="7.8" cy="15.4" rx="3" ry="2.6" />
    <ellipse cx="16.2" cy="15.4" rx="3" ry="2.6" />
  </svg>
);

const ICO =
  "grid h-[30px] w-[30px] shrink-0 place-items-center rounded-md text-neutral-500 transition dark:text-[#8B92A1] " +
  "hover:bg-black/[0.045] hover:text-inherit dark:hover:bg-white/[0.07] " +
  "focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-[#2F62D8] dark:focus-visible:outline-[#6E9BFF]";

const KBD =
  "inline-grid h-5 min-w-5 shrink-0 place-items-center rounded px-1 font-sans text-[11.5px] leading-none " +
  "tabular-nums bg-black/[0.06] text-neutral-500 dark:bg-white/[0.08] dark:text-[#A2A9B8]";

export default function Picker() {
  const [incoming, setIncoming] = useState<Incoming | null>(null);
  const [rows, setRows] = useState<Destination[]>([]);
  const [remember, setRemember] = useState<Remember>("once");
  const [priv, setPriv] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [cursor, setCursor] = useState(0);

  const frame = useRef<HTMLDivElement>(null);
  const rowRefs = useRef<(HTMLButtonElement | null)[]>([]);

  useEffect(() => {
    void invoke<Destinations>("picker_destinations").then((d) => setRows(rowsOf(d.browsers)));
  }, []);

  const arrive = useCallback((next: Incoming) => {
    setIncoming(next);
    setProblem(null);
    setCopied(false);
    setPriv(false);
    setRemember("once");
    setCursor(0);
  }, []);

  useEffect(() => {
    void invoke<Incoming | null>("picker_boot").then((pending) => {
      if (pending) arrive(pending);
    });
    const stop = listen<Incoming>("link:incoming", (e) => arrive(e.payload));
    return () => {
      void stop.then((f) => f());
    };
  }, [arrive]);

  useLayoutEffect(() => {
    if (!frame.current || rows.length === 0) return;
    const report = () => {
      const height = frame.current?.getBoundingClientRect().height;
      if (height) void invoke("picker_fit", { height });
    };
    report();
    const watch = new ResizeObserver(report);
    watch.observe(frame.current);
    return () => watch.disconnect();
    // `incoming` is a dependency: a second link with the same row count leaves the
    // height untouched, so the observer alone would never show the window again.
  }, [rows.length, incoming]);

  useEffect(() => {
    rowRefs.current[cursor]?.focus();
  }, [cursor]);

  const open = useCallback(
    (row: Destination, isPrivate: boolean) => {
      void invoke("picker_open", {
        browserId: row.browser.id,
        profileId: row.profile?.id ?? null,
        private: isPrivate,
        remember,
      }).catch((e: unknown) => setProblem(String(e)));
    },
    [remember],
  );

  const host = incoming?.host ?? "";
  const site = incoming?.site ?? "";
  const url = incoming?.url ?? "";
  const from = incoming?.source_app;
  const { head, tail } = split(url);

  // A host that is already its own site would save the very same rule twice.
  const scopes: { id: Remember; label: string; keeps: boolean; dead: boolean }[] = [
    { id: "once", label: "Solo esta vez", keeps: false, dead: false },
    { id: "url", label: "Esta URL", keeps: true, dead: false },
    { id: "host", label: "Subdominio", keeps: true, dead: host === site || host === "" },
    { id: "site", label: "Todo el sitio", keeps: true, dead: false },
  ];

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        void invoke("picker_dismiss");
        return;
      }
      const index = Number.parseInt(e.key, 10) - 1;
      if (index >= 0 && index < rows.length && !e.ctrlKey && !e.altKey) {
        e.preventDefault();
        open(rows[index], e.shiftKey);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [rows, open]);

  const onListKey = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const step = e.key === "ArrowDown" ? 1 : -1;
      setCursor((c) => (c + step + rows.length) % rows.length);
    }
  };

  return (
    <div
      ref={frame}
      className="flex flex-col overflow-hidden rounded-[10px] border border-black/10 bg-white text-[#16181D] shadow-[0_16px_40px_rgba(20,25,45,0.2)] dark:border-white/10 dark:bg-[#191A1F] dark:text-[#F0F1F4] dark:shadow-[0_16px_40px_rgba(0,0,0,0.5)]"
    >
      <div className="flex items-center gap-2 border-b border-black/[0.08] py-[13px] pr-[11px] pl-[15px] dark:border-white/[0.08]">
        <div className="flex min-w-0 flex-1 flex-col gap-1">
          <p className="truncate text-[13.5px] leading-[18px]">
            <b className="font-semibold">{head}</b>
            <span className="text-neutral-400 dark:text-[#646B7C]">{tail}</span>
          </p>
          {from && (
            <p className="truncate text-[11.5px] leading-[15px] text-neutral-500 dark:text-[#8B92A1]">
              Desde <b className="font-semibold">{from}</b>
            </p>
          )}
        </div>
        <button
          type="button"
          title={copied ? "Copiado" : "Copiar URL"}
          aria-label="Copiar URL"
          onClick={() => {
            void navigator.clipboard.writeText(url);
            setCopied(true);
          }}
          className={ICO}
        >
          {COPY}
        </button>
        <button
          type="button"
          title="Ventana privada"
          aria-label="Ventana privada"
          aria-pressed={priv}
          onClick={() => setPriv((p) => !p)}
          className={
            priv
              ? `${ICO} bg-[#2F62D8]/[0.13] text-[#2F62D8] dark:bg-[#6E9BFF]/[0.17] dark:text-[#6E9BFF]`
              : ICO
          }
        >
          {MASK}
        </button>
      </div>

      <div
        role="menu"
        aria-label="Abrir en"
        className="flex flex-col px-1.5 py-[7px]"
        onKeyDown={onListKey}
      >
        {rows.map((row, i) => (
          <button
            key={`${row.browser.id}-${row.profile?.id ?? "default"}`}
            ref={(node) => {
              rowRefs.current[i] = node;
            }}
            type="button"
            role="menuitem"
            tabIndex={i === cursor ? 0 : -1}
            onFocus={() => setCursor(i)}
            onClick={(e) => open(row, priv || e.shiftKey)}
            className="flex h-[38px] w-full items-center gap-[11px] rounded-[7px] px-2 text-left transition hover:bg-black/[0.045] focus-visible:bg-black/[0.045] focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-[#2F62D8] dark:hover:bg-white/[0.07] dark:focus-visible:bg-white/[0.07] dark:focus-visible:outline-[#6E9BFF]"
          >
            <kbd className={KBD}>{i + 1}</kbd>
            {row.browser.icon ? (
              <img src={row.browser.icon} alt="" className="h-5 w-5 shrink-0" />
            ) : (
              <span className="h-5 w-5 shrink-0 rounded bg-black/10 dark:bg-white/10" />
            )}
            <span className="min-w-0 shrink truncate text-[13.5px]">{row.browser.name}</span>
            {row.profile && (
              <span className="shrink-0 rounded bg-black/[0.05] px-2 py-0.5 text-[11.5px] leading-4 text-neutral-500 dark:bg-white/[0.065] dark:text-[#8B92A1]">
                {row.profile.name}
              </span>
            )}
          </button>
        ))}
      </div>

      {problem && (
        <p className="mx-[15px] mb-1.5 rounded-md bg-[#C4433F]/10 px-2.5 py-2 text-[11.5px] text-[#C4433F] dark:bg-[#FF8A85]/10 dark:text-[#FF8A85]">
          {problem} — elige otro destino.
        </p>
      )}

      <div className="flex flex-col border-t border-black/[0.08] dark:border-white/[0.08]">
        <div className="flex h-[38px] items-center gap-2 px-[15px] text-[12px] text-neutral-400 dark:text-[#646B7C]">
          <span>Recordar esta elección</span>
          <span className="flex-1" />
          <span className="flex shrink-0 items-center gap-1.5 text-[11.5px]">
            <kbd className={KBD}>Mayús</kbd>
            privada
          </span>
        </div>
        <fieldset className="flex gap-1 px-[15px] pb-3">
          <legend className="sr-only">Recordar esta elección</legend>
          {scopes.map((s) => {
            const on = s.id === remember;
            const tone = !on
              ? "bg-black/[0.045] text-neutral-500 hover:text-inherit dark:bg-white/[0.05] dark:text-[#8B92A1]"
              : s.keeps
                ? "bg-[#2F62D8]/[0.13] font-semibold text-[#2F62D8] dark:bg-[#6E9BFF]/[0.17] dark:text-[#6E9BFF]"
                : "bg-black/[0.09] font-semibold text-[#3A3F4B] dark:bg-white/[0.11] dark:text-[#D6DAE3]";
            return (
              <label
                key={s.id}
                className="contents"
                title={s.dead ? "Este host no tiene subdominio propio" : undefined}
              >
                <input
                  type="radio"
                  name="remember"
                  className="peer sr-only"
                  checked={on}
                  disabled={s.dead}
                  onChange={() => setRemember(s.id)}
                />
                <span
                  className={`grid h-7 min-w-0 flex-1 cursor-pointer place-items-center truncate rounded-md px-1.5 text-[11.5px] leading-none transition peer-disabled:cursor-default peer-disabled:opacity-40 peer-focus-visible:outline-2 peer-focus-visible:outline-offset-1 peer-focus-visible:outline-[#2F62D8] dark:peer-focus-visible:outline-[#6E9BFF] ${tone}`}
                >
                  {s.label}
                </span>
              </label>
            );
          })}
        </fieldset>
      </div>
    </div>
  );
}
