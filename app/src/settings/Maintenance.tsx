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
        <p className="rounded-md bg-danger/10 px-3 py-2 text-sm text-danger">{problem}</p>
      )}
      {done && (
        <p className="rounded-md bg-good/10 px-3 py-2 text-sm text-good">
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
              className="shrink-0 rounded-md px-3 py-1.5 text-sm text-danger"
            >
              {t("resetGo")}
            </button>
          </Line>
          <Line title={t("unregisterTitle")} note={t("unregisterNote")}>
            <button
              type="button"
              onClick={() => setAsking("unregister")}
              className="shrink-0 rounded-md px-3 py-1.5 text-sm text-danger"
            >
              {t("unregisterGo")}
            </button>
          </Line>
        </Card>
      </Section>
    </>
  );
}
