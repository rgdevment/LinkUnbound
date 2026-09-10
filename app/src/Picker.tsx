import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";

type Profile = { id: string; name: string };
type Browser = {
  id: string;
  name: string;
  exe: string;
  profiles: Profile[];
  private_flag: string | null;
};
type Destinations = { browsers: Browser[]; is_default: boolean };
type Incoming = { url: string; source_app: string | null };

/// One row per destination: a browser without profiles is one row, a browser
/// with them is one row each, because that is the choice being made.
type Destination = { browser: Browser; profile: Profile | null };

function rowsOf(browsers: Browser[]): Destination[] {
  return browsers.flatMap((browser): Destination[] =>
    browser.profiles.length === 0
      ? [{ browser, profile: null }]
      : browser.profiles.map((profile) => ({ browser, profile })),
  );
}

function hostOf(url: string): string {
  try {
    return new URL(url).host;
  } catch {
    return url;
  }
}

export default function Picker() {
  const [incoming, setIncoming] = useState<Incoming | null>(null);
  const [rows, setRows] = useState<Destination[]>([]);
  const [remember, setRemember] = useState(false);
  const [priv, setPriv] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);

  useEffect(() => {
    void invoke<Destinations>("picker_destinations").then((d) => setRows(rowsOf(d.browsers)));
  }, []);

  useEffect(() => {
    void invoke<Incoming | null>("picker_boot").then((pending) => {
      if (pending) setIncoming(pending);
    });
    const stop = listen<Incoming>("link:incoming", (e) => {
      setIncoming(e.payload);
      setProblem(null);
    });
    return () => {
      void stop.then((f) => f());
    };
  }, []);

  const open = useCallback(
    (row: Destination) => {
      void invoke("picker_open", {
        browserId: row.browser.id,
        profileId: row.profile?.id ?? null,
        private: priv,
        remember,
      }).catch((e: unknown) => setProblem(String(e)));
    },
    [priv, remember],
  );

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      setPriv(e.shiftKey);
      if (e.key === "Escape") void invoke("picker_dismiss");
      const index = Number.parseInt(e.key, 10) - 1;
      if (e.type === "keydown" && index >= 0 && index < rows.length) open(rows[index]);
    };
    window.addEventListener("keydown", onKey);
    window.addEventListener("keyup", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("keyup", onKey);
    };
  }, [rows, open]);

  const url = incoming?.url ?? "";

  return (
    <div className="flex h-full flex-col rounded-xl border border-[#313244] bg-[#1E1E2E] p-[5px] text-[#CDD6F4] shadow-2xl">
      <div className="flex items-start gap-2 px-3 pt-3 pb-2.5">
        <div className="min-w-0 flex-grow">
          <p className="truncate text-[13px] font-semibold">{hostOf(url)}</p>
          <p className="truncate text-[11px] text-[#A6ADC8]">{url}</p>
        </div>
        <button
          type="button"
          onClick={() => void navigator.clipboard.writeText(url)}
          className="shrink-0 rounded-md bg-[#313244] px-2 py-1 text-[11px] text-[#A6ADC8] hover:text-[#CDD6F4]"
        >
          Copiar
        </button>
        <button
          type="button"
          onClick={() => setPriv((p) => !p)}
          className={`shrink-0 rounded-full px-2.5 py-1 text-[11px] ${
            priv ? "bg-[#45375C] text-[#CBA6F7]" : "bg-[#313244] text-[#A6ADC8]"
          }`}
        >
          Privada
        </button>
      </div>

      <div className="flex flex-col gap-0.5 px-[5px]">
        {rows.map((row, i) => (
          <button
            key={`${row.browser.id}-${row.profile?.id ?? "default"}`}
            type="button"
            onClick={() => open(row)}
            className="flex items-center gap-2.5 rounded-lg px-2.5 py-2 text-left hover:bg-[#313244] focus:bg-[#313244] focus:outline-none"
          >
            <span className="h-5 w-5 shrink-0 rounded-md bg-[#585B70]" />
            <span className="flex-grow text-[13px]">{row.browser.name}</span>
            {row.profile && (
              <span className="rounded-full bg-[#313244] px-1.5 py-0.5 text-[11px] text-[#A6ADC8]">
                {row.profile.name}
              </span>
            )}
            <span className="text-[10.5px] text-[#6C7086]">{i + 1}</span>
          </button>
        ))}
      </div>

      {problem && (
        <p className="px-3 py-2 text-[11px] text-[#F38BA8]">{problem} — elige otro destino.</p>
      )}

      <div className="mt-1.5 flex items-center gap-2 border-t border-[#313244] px-3 py-2.5">
        <input
          id="remember"
          type="checkbox"
          checked={remember}
          onChange={(e) => setRemember(e.target.checked)}
        />
        <label htmlFor="remember" className="text-[12px] text-[#BAC2DE]">
          Abrir siempre aquí los enlaces de{" "}
          <span className="font-semibold text-[#CDD6F4]">
            {incoming?.source_app ?? hostOf(url)}
          </span>
        </label>
      </div>
    </div>
  );
}
