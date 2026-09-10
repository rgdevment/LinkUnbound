import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useLayoutEffect, useRef, useState } from "react";

type Fired = { url: string; rule_id: string; browser: string };

const SECONDS = 6;

function hostOf(url: string): string {
  try {
    return new URL(url).host;
  } catch {
    return url;
  }
}

export default function Notice() {
  const [fired, setFired] = useState<Fired | null>(null);
  const [left, setLeft] = useState(SECONDS);

  const frame = useRef<HTMLDivElement>(null);

  useEffect(() => {
    void invoke<Fired | null>("notice_boot").then((pending) => {
      if (pending) {
        setFired(pending);
        setLeft(SECONDS);
      }
    });
    const stop = listen<Fired>("rule:fired", (e) => {
      setFired(e.payload);
      setLeft(SECONDS);
    });
    return () => {
      void stop.then((f) => f());
    };
  }, []);

  useLayoutEffect(() => {
    if (fired) void invoke("notice_ready");
  }, [fired]);

  useEffect(() => {
    if (!fired) return;
    if (left <= 0) {
      void invoke("notice_dismiss");
      return;
    }
    const tick = setTimeout(() => setLeft((n) => n - 1), 1000);
    return () => clearTimeout(tick);
  }, [fired, left]);

  if (!fired) return null;

  return (
    <div
      ref={frame}
      className="flex h-full items-center gap-3 overflow-hidden rounded-[10px] border border-black/10 bg-white px-3.5 text-[#16181D] shadow-[0_16px_40px_rgba(20,25,45,0.2)] dark:border-white/10 dark:bg-[#191A1F] dark:text-[#F0F1F4] dark:shadow-[0_16px_40px_rgba(0,0,0,0.5)]"
    >
      <div className="min-w-0 flex-1">
        <p className="truncate text-[12.5px]">
          Abierto en <b className="font-semibold">{fired.browser}</b>
        </p>
        <p className="truncate text-[11px] text-neutral-500 dark:text-[#8B92A1]">
          Una regla decidió por {hostOf(fired.url)}
        </p>
      </div>
      <button
        type="button"
        onClick={() => void invoke("notice_undo", { ruleId: fired.rule_id })}
        className="shrink-0 rounded-md bg-black/[0.06] px-3 py-1.5 text-[11.5px] font-medium dark:bg-white/[0.09]"
      >
        Deshacer
      </button>
      <button
        type="button"
        aria-label="Cerrar el aviso"
        onClick={() => void invoke("notice_dismiss")}
        className="grid h-6 w-6 shrink-0 place-items-center rounded text-neutral-500 tabular-nums dark:text-[#8B92A1]"
      >
        {left}
      </button>
    </div>
  );
}
