import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { type Key, useSpoken, useWords } from "../i18n";
import { saidPlainly } from "../refusal";
import Confirm from "./Confirm";
import { Card, Line, Section } from "./parts";

type Pending = "reset" | "unregister" | null;

const ASKS: Record<string, { title: Key; body: Key; go: Key }> = {
  reset: { title: "resetTitle", body: "resetBody", go: "resetGo" },
  unregister: { title: "unregisterTitle", body: "unregisterBody", go: "unregisterGo" },
};

export default function Maintenance() {
  const t = useWords();
  const language = useSpoken();
  const [asking, setAsking] = useState<Pending>(null);
  const [done, setDone] = useState<string | null>(null);
  const [problem, setProblem] = useState<string | null>(null);

  const confirm = () => {
    if (!asking) return;
    const command = asking === "reset" ? "maintenance_reset" : "system_set_registered";
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
        <Confirm
          title={t(ASKS[asking].title)}
          body={t(ASKS[asking].body)}
          go={t(ASKS[asking].go)}
          onConfirm={confirm}
          onCancel={() => setAsking(null)}
        />
      )}

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
