import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { Card, Line, Section } from "./parts";

type Pending = "rescan" | "reset" | "unregister" | null;

const ASKS: Record<string, { title: string; body: string; go: string }> = {
  rescan: {
    title: "Volver a buscar navegadores",
    body: "Se olvida lo detectado y se vuelve a leer del registro. Los navegadores que añadiste a mano se conservan; lo que hayas ocultado se mostrará de nuevo.",
    go: "Buscar",
  },
  reset: {
    title: "Restablecer la configuración",
    body: "Se borran todas las reglas y todos los navegadores, incluidos los que añadiste. No se puede deshacer.",
    go: "Restablecer",
  },
  unregister: {
    title: "Quitar LinkUnbound de Windows",
    body: "Deja de ofrecerse como navegador. Es posible que después tengas que elegir otro predeterminado en la configuración de Windows.",
    go: "Quitar",
  },
};

export default function Maintenance() {
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
      .catch((e: unknown) => setProblem(String(e)));
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
        setDone(ASKS[asking].title);
        setProblem(null);
      })
      .catch((e: unknown) => setProblem(String(e)))
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
          Hecho: {done.toLowerCase()}.
        </p>
      )}

      {asking && (
        <div
          role="alertdialog"
          aria-label={ASKS[asking].title}
          className="rounded-lg border border-black/[0.12] bg-black/[0.02] p-3.5 dark:border-white/[0.12] dark:bg-white/[0.03]"
        >
          <p className="text-[12.5px] font-semibold">{ASKS[asking].title}</p>
          <p className="mt-1 text-[11.5px] text-neutral-500 dark:text-[#8B92A1]">
            {ASKS[asking].body}
          </p>
          <div className="mt-3 flex gap-2">
            <button
              type="button"
              onClick={confirm}
              className="rounded-md bg-[#C0362F] px-3 py-1.5 text-[11.5px] font-medium text-white dark:bg-[#FF8A85] dark:text-[#12141B]"
            >
              {ASKS[asking].go}
            </button>
            <button
              type="button"
              onClick={() => setAsking(null)}
              className="rounded-md px-3 py-1.5 text-[11.5px] text-neutral-500 dark:text-[#8B92A1]"
            >
              Cancelar
            </button>
          </div>
        </div>
      )}

      <Section title="Informar de un problema">
        <Card>
          <Line
            title="Guardar un informe de diagnóstico"
            note="Un archivo con el estado de la app y tus reglas, sin las direcciones que visitas"
          >
            <button
              type="button"
              onClick={report}
              className="shrink-0 rounded-md border border-black/[0.12] px-3 py-1.5 text-[11.5px] dark:border-white/[0.12]"
            >
              Guardar
            </button>
          </Line>
        </Card>
        {saved && (
          <p className="rounded-md bg-[#1E7A52]/10 px-3 py-2 text-[11.5px] break-all text-[#1E7A52] dark:bg-[#4CC38A]/10 dark:text-[#4CC38A]">
            Guardado en {saved}
          </p>
        )}
      </Section>

      <Section title="Navegadores">
        <Card>
          <Line
            title="Volver a buscar navegadores"
            note="Útil si instalaste uno y no aparece en el selector"
          >
            <button
              type="button"
              onClick={() => setAsking("rescan")}
              className="shrink-0 rounded-md border border-black/[0.12] px-3 py-1.5 text-[11.5px] dark:border-white/[0.12]"
            >
              Buscar
            </button>
          </Line>
        </Card>
      </Section>

      <Section title="Empezar de cero">
        <Card>
          <Line
            title="Restablecer la configuración"
            note="Borra todas las reglas y todos los navegadores"
          >
            <button
              type="button"
              onClick={() => setAsking("reset")}
              className="shrink-0 rounded-md px-3 py-1.5 text-[11.5px] text-[#C0362F] dark:text-[#FF8A85]"
            >
              Restablecer
            </button>
          </Line>
          <Line
            title="Quitar LinkUnbound de Windows"
            note="Deja de ofrecerse como navegador en la lista del sistema"
          >
            <button
              type="button"
              onClick={() => setAsking("unregister")}
              className="shrink-0 rounded-md px-3 py-1.5 text-[11.5px] text-[#C0362F] dark:text-[#FF8A85]"
            >
              Quitar
            </button>
          </Line>
        </Card>
      </Section>
    </>
  );
}
