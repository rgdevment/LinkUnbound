import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useWords } from "../i18n";

const REPO = "https://github.com/rgdevment/LinkUnbound";

export default function Star({ onSettled }: { onSettled: () => void }) {
  const t = useWords();

  const settle = (open: boolean) => {
    onSettled();
    void invoke("star_done").catch(noop);
    if (open) void openUrl(REPO).catch(noop);
  };

  return (
    <div
      role="status"
      className="flex items-start gap-3 rounded-lg border border-black/[0.08] bg-black/[0.02] px-3.5 py-3 dark:border-white/[0.08] dark:bg-white/[0.03]"
    >
      <svg
        viewBox="0 0 16 16"
        aria-hidden="true"
        className="mt-px h-[15px] w-[15px] shrink-0 text-[#C8892A] dark:text-[#E3B341]"
      >
        <path
          fill="currentColor"
          d="M8 1.2l2.1 4.3 4.7.7-3.4 3.3.8 4.7L8 12l-4.2 2.2.8-4.7L1.2 6.2l4.7-.7L8 1.2z"
        />
      </svg>
      <div className="min-w-0 flex-1">
        <p className="text-[12.5px] font-medium">{t("starThanks")}</p>
        <p className="mt-0.5 text-[11.5px] leading-relaxed text-neutral-600 dark:text-[#98A0B4]">
          {t("starWhy")}
        </p>
        <div className="mt-2.5 flex flex-wrap gap-2">
          <button
            type="button"
            onClick={() => settle(true)}
            className="rounded-md bg-[#2F62D8] px-3 py-1.5 text-[11.5px] font-medium text-white dark:bg-[#6E9BFF] dark:text-[#12141B]"
          >
            {t("starGo")}
          </button>
          <button
            type="button"
            onClick={onSettled}
            className="rounded-md border border-black/[0.12] px-3 py-1.5 text-[11.5px] dark:border-white/[0.12]"
          >
            {t("starLater")}
          </button>
          <button
            type="button"
            onClick={() => settle(false)}
            className="rounded-md px-3 py-1.5 text-[11.5px] text-neutral-500 dark:text-[#8B92A1]"
          >
            {t("starNo")}
          </button>
        </div>
      </div>
    </div>
  );
}

function noop() {}
