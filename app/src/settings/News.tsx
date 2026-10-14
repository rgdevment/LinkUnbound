import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useCallback, useEffect, useRef, useState } from "react";
import bear from "../assets/linkunbound.svg";
import { useSpoken, useWords } from "../i18n";
import { headline, toldFor } from "./told";

const RELEASES = "https://github.com/rgdevment/LinkUnbound/releases";

export default function News() {
  const t = useWords();
  const language = useSpoken();
  const [versions, setVersions] = useState<string[]>([]);
  const ok = useRef<HTMLButtonElement>(null);
  const box = useRef<HTMLDivElement>(null);

  useEffect(() => {
    void invoke<string[]>("news")
      .then((found) => setVersions(Array.isArray(found) ? found : []))
      .catch(noop);
  }, []);

  const told = toldFor(versions, language);
  const open = told.length > 0;

  const close = useCallback(() => {
    setVersions([]);
    void invoke("news_seen").catch(noop);
  }, []);

  useEffect(() => {
    if (!open) return;
    ok.current?.focus();
    const held = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        close();
        return;
      }
      if (e.key !== "Tab" || !box.current) return;
      const reachable = [...box.current.querySelectorAll<HTMLElement>("button")];
      const first = reachable[0];
      const last = reachable[reachable.length - 1];
      const at = document.activeElement;
      const outside = !box.current.contains(at);
      if (e.shiftKey && (at === first || outside)) {
        e.preventDefault();
        last?.focus();
      } else if (!e.shiftKey && (at === last || outside)) {
        e.preventDefault();
        first?.focus();
      }
    };
    document.addEventListener("keydown", held);
    return () => document.removeEventListener("keydown", held);
  }, [open, close]);

  if (!open) return null;
  const title = headline(versions[0], language) ?? t("newsTitle");

  return (
    <div className="fixed inset-0 z-50 grid place-items-center bg-black/40 p-6">
      <div
        ref={box}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        className="w-full max-w-md rounded-[10px] border border-black/[0.08] bg-white p-6 shadow-xl dark:border-white/[0.08] dark:bg-[#1E2027]"
      >
        <div className="mb-3 flex flex-col items-center gap-2">
          <img src={bear} alt="" className="bear h-[64px]" />
          <span className="flex items-center gap-2 text-[10.5px] font-semibold tracking-[0.06em] text-neutral-500 uppercase dark:text-[#8B92A1]">
            {t("newsOver")}
            <span className="rounded-full bg-[#2F62D8]/10 px-2 py-0.5 font-mono tracking-normal text-[#2F62D8] normal-case dark:bg-[#6E9BFF]/15 dark:text-[#6E9BFF]">
              {versions[0]}
            </span>
          </span>
        </div>
        <h2 className="mb-3 text-center text-[17px] font-semibold">{title}</h2>
        <div className="scroller -mr-3 max-h-[46vh] pr-3">
          {told.map((one, at) => (
            <section key={one.version} aria-label={one.version}>
              {at > 0 && (
                <h3 className="mt-4 mb-1 font-mono text-[11px] text-neutral-500 dark:text-[#8B92A1]">
                  {one.version}
                </h3>
              )}
              <ul>
                {one.told.map((said) => (
                  <li
                    key={`${one.version}-${said.title}`}
                    className="relative border-b border-black/[0.08] py-2.5 pl-4 text-[12px] leading-relaxed text-neutral-600 last:border-b-0 before:absolute before:top-[16px] before:left-0 before:size-1.5 before:rounded-full before:bg-[#2F62D8] dark:border-white/[0.08] dark:text-[#98A0B4] dark:before:bg-[#6E9BFF]"
                  >
                    <b className="font-semibold text-[#16181D] dark:text-[#F0F1F4]">{said.title}</b>
                    : {said.said}
                  </li>
                ))}
              </ul>
            </section>
          ))}
        </div>
        <div className="mt-4 flex items-center gap-4">
          <button
            type="button"
            onClick={() => void openUrl(RELEASES).catch(noop)}
            className="text-[12px] text-[#2F62D8] hover:underline dark:text-[#6E9BFF]"
          >
            {t("newsAll")}
          </button>
          <button
            type="button"
            ref={ok}
            onClick={close}
            className="ml-auto rounded-md bg-[#2F62D8] px-3.5 py-1.5 text-[12px] font-medium text-white dark:bg-[#6E9BFF] dark:text-[#12141B]"
          >
            {t("newsOk")}
          </button>
        </div>
      </div>
    </div>
  );
}

function noop() {}
