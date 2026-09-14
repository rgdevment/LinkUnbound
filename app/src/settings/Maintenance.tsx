import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { type Key, useSpoken, useWords } from "../i18n";
import { saidPlainly } from "../refusal";
import { Card, Line, Section } from "./parts";

type Pending = "rescan" | "reset" | "unregister" | null;

const ASKS: Record<string, { title: Key; body: Key; go: Key }> = {
  rescan: { title: "rescanTitle", body: "rescanBody", go: "rescanGo" },
  reset: { title: "resetTitle", body: "resetBody", go: "resetGo" },
  unregister: { title: "unregisterTitle", body: "unregisterBody", go: "unregisterGo" },
};

export default function Maintenance() {
  const t = useWords();
  const language = useSpoken();
  const [asking, setAsking] = useState<Pending>(null);
  const [done, setDone] = useState<string | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [saved, setSaved] = useState<string | null>(null);

  const report = () => {
    void invoke<string>("maintenance_report")
      .then((path) => {
        setSaved(path);
        setProblem(null);
      })
      .catch((e: unknown) => setProblem(saidPlainly(language, e)));
  };

  const confirm = () => {
    if (!asking) return;
    const command =
      asking === "reset"
        ? "maintenance_reset"
        : asking === "rescan"
          ? "maintenance_rescan"
          : "system_set_registered";
    const params = asking === "unregister" ? { enabled: false } : {};
    void invoke(command, params)
      .then(() => {
        setDone(t(ASKS[asking].title));
        setProblem(null);
      })
      .catch((e: unknown) => setProblem(saidPlainly(language, e)))
      .finally(() => setAsking(null));
  };

  return (
    <>
      {problem && (
        <p className="rounded-md bg-[#C0362F]/10 px-3 py-2 text-[11.5px] text-[#C0362F] dark:bg-[#FF8A85]/10 dark:text-[#FF8A85]">
          {problem}
        </p>
      )}
      {done && (
        <p className="rounded-md bg-[#1E7A52]/10 px-3 py-2 text-[11.5px] text-[#1E7A52] dark:bg-[#4CC38A]/10 dark:text-[#4CC38A]">
          {t("doneWith", done.toLowerCase())}
        </p>
      )}

      {asking && (
        <div
          role="alertdialog"
          aria-label={t(ASKS[asking].title)}
          className="rounded-lg border border-black/[0.12] bg-black/[0.02] p-3.5 dark:border-white/[0.12] dark:bg-white/[0.03]"
        >
          <p className="text-[12.5px] font-semibold">{t(ASKS[asking].title)}</p>
          <p className="mt-1 text-[11.5px] text-neutral-500 dark:text-[#8B92A1]">
            {t(ASKS[asking].body)}
          </p>
          <div className="mt-3 flex gap-2">
            <button
              type="button"
              onClick={confirm}
              className="rounded-md bg-[#C0362F] px-3 py-1.5 text-[11.5px] font-medium text-white dark:bg-[#FF8A85] dark:text-[#12141B]"
            >
              {t(ASKS[asking].go)}
            </button>
            <button
              type="button"
              onClick={() => setAsking(null)}
              className="rounded-md px-3 py-1.5 text-[11.5px] text-neutral-500 dark:text-[#8B92A1]"
            >
              {t("cancel")}
            </button>
          </div>
        </div>
      )}

      <Section title={t("careReport")}>
        <Card>
          <Line title={t("reportTitle")} note={t("reportNote")}>
            <button
              type="button"
              onClick={report}
              className="shrink-0 rounded-md border border-black/[0.12] px-3 py-1.5 text-[11.5px] dark:border-white/[0.12]"
            >
              {t("save")}
            </button>
          </Line>
        </Card>
        {saved && (
          <p className="rounded-md bg-[#1E7A52]/10 px-3 py-2 text-[11.5px] break-all text-[#1E7A52] dark:bg-[#4CC38A]/10 dark:text-[#4CC38A]">
            {t("reportSaved", saved)}
          </p>
        )}
      </Section>

      <Section title={t("careBrowsers")}>
        <Card>
          <Line title={t("rescanTitle")} note={t("rescanNote")}>
            <button
              type="button"
              onClick={() => setAsking("rescan")}
              className="shrink-0 rounded-md border border-black/[0.12] px-3 py-1.5 text-[11.5px] dark:border-white/[0.12]"
            >
              {t("rescanGo")}
            </button>
          </Line>
        </Card>
      </Section>

      <Section title={t("careScratch")}>
        <Card>
          <Line title={t("resetTitle")} note={t("resetNote")}>
            <button
              type="button"
              onClick={() => setAsking("reset")}
              className="shrink-0 rounded-md px-3 py-1.5 text-[11.5px] text-[#C0362F] dark:text-[#FF8A85]"
            >
              {t("resetGo")}
            </button>
          </Line>
          <Line title={t("unregisterTitle")} note={t("unregisterNote")}>
            <button
              type="button"
              onClick={() => setAsking("unregister")}
              className="shrink-0 rounded-md px-3 py-1.5 text-[11.5px] text-[#C0362F] dark:text-[#FF8A85]"
            >
              {t("unregisterGo")}
            </button>
          </Line>
        </Card>
      </Section>
    </>
  );
}
