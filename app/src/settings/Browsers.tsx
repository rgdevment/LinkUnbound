import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Card, Section, Switch } from "./parts";

type BrowserView = {
  id: string;
  name: string;
  exe: string;
  profiles: number;
  private: boolean;
  custom: boolean;
  hidden: boolean;
  icon: string | null;
};

function describe(browser: BrowserView): string {
  const parts = [
    browser.profiles === 0
      ? "Sin perfiles"
      : browser.profiles === 1
        ? "1 perfil"
        : `${browser.profiles} perfiles`,
  ];
  if (browser.private) parts.push("admite ventana privada");
  if (browser.hidden) parts.push("oculto del selector");
  return parts.join(" · ");
}

function Icon({ browser }: { browser: BrowserView }) {
  if (browser.icon) return <img src={browser.icon} alt="" className="h-[18px] w-[18px] shrink-0" />;
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      className="h-[18px] w-[18px] shrink-0 text-neutral-400 dark:text-[#646B7C]"
      aria-hidden="true"
    >
      <rect x="3" y="4" width="18" height="16" rx="3" />
      <path d="M3 9h18" />
    </svg>
  );
}

export default function Browsers() {
  const [list, setList] = useState<BrowserView[] | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [adding, setAdding] = useState(false);
  const [name, setName] = useState("");
  const [exe, setExe] = useState("");
  const [args, setArgs] = useState("");

  const run = (command: string, params: Record<string, unknown> = {}) => {
    void invoke<BrowserView[]>(command, params)
      .then((next) => {
        setList(next);
        setProblem(null);
        setAdding(false);
      })
      .catch((e: unknown) => setProblem(String(e)));
  };

  useEffect(() => run("browsers_list"), []);

  if (list === null) return null;

  const detected = list.filter((b) => !b.custom);
  const mine = list.filter((b) => b.custom);

  return (
    <>
      {problem && (
        <p className="rounded-md bg-[#C0362F]/10 px-3 py-2 text-[11.5px] text-[#C0362F] dark:bg-[#FF8A85]/10 dark:text-[#FF8A85]">
          {problem}
        </p>
      )}

      <Section title="Detectados en el equipo">
        <Card>
          {detected.map((b) => (
            <div
              key={b.id}
              className="flex items-center gap-3 border-black/[0.08] px-3.5 py-3 not-first:border-t dark:border-white/[0.08]"
            >
              <Icon browser={b} />
              <div className="min-w-0 flex-1">
                <p className={`text-[12.5px] ${b.hidden ? "opacity-55" : ""}`}>{b.name}</p>
                <p className="mt-px text-[11px] text-neutral-500 dark:text-[#8B92A1]">
                  {describe(b)}
                </p>
              </div>
              <Switch
                on={!b.hidden}
                label={`Mostrar ${b.name} en el selector`}
                onChange={(next) => run("browsers_set_hidden", { id: b.id, hidden: !next })}
              />
            </div>
          ))}
          {detected.length === 0 && (
            <p className="px-3.5 py-4 text-center text-[12px] text-neutral-500 dark:text-[#8B92A1]">
              Windows no reporta ningún navegador instalado.
            </p>
          )}
        </Card>
      </Section>

      <Section title="Añadidos por ti">
        <Card>
          {mine.map((b) => (
            <div
              key={b.id}
              className="flex items-center gap-3 border-black/[0.08] px-3.5 py-3 not-first:border-t dark:border-white/[0.08]"
            >
              <Icon browser={b} />
              <div className="min-w-0 flex-1">
                <p className="truncate text-[12.5px]">{b.name}</p>
                <p className="mt-px truncate text-[11px] text-neutral-500 dark:text-[#8B92A1]">
                  {b.exe}
                </p>
              </div>
              <button
                type="button"
                aria-label={`Eliminar ${b.name}`}
                onClick={() => run("browsers_remove", { id: b.id })}
                className="grid h-6 w-6 shrink-0 place-items-center rounded text-neutral-500 hover:bg-[#C0362F]/10 hover:text-[#C0362F] dark:text-[#8B92A1] dark:hover:bg-[#FF8A85]/10 dark:hover:text-[#FF8A85]"
              >
                ✕
              </button>
            </div>
          ))}

          {adding ? (
            <form
              className="flex flex-col gap-2 border-black/[0.08] px-3.5 py-3 not-first:border-t dark:border-white/[0.08]"
              onSubmit={(e) => {
                e.preventDefault();
                run("browsers_add", {
                  name,
                  exe,
                  args: args.split(" ").filter(Boolean),
                });
              }}
            >
              <input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Nombre"
                aria-label="Nombre"
                required
                className="rounded-md border border-black/[0.12] bg-transparent px-2.5 py-1.5 text-[12px] dark:border-white/[0.12]"
              />
              <input
                value={exe}
                onChange={(e) => setExe(e.target.value)}
                placeholder="Ruta del ejecutable"
                aria-label="Ruta del ejecutable"
                required
                className="rounded-md border border-black/[0.12] bg-transparent px-2.5 py-1.5 text-[12px] dark:border-white/[0.12]"
              />
              <input
                value={args}
                onChange={(e) => setArgs(e.target.value)}
                placeholder="Argumentos adicionales (separados por espacios)"
                aria-label="Argumentos adicionales"
                className="rounded-md border border-black/[0.12] bg-transparent px-2.5 py-1.5 text-[12px] dark:border-white/[0.12]"
              />
              <div className="flex gap-2">
                <button
                  type="submit"
                  className="rounded-md bg-[#2F62D8] px-3 py-1.5 text-[11.5px] font-medium text-white dark:bg-[#6E9BFF] dark:text-[#12141B]"
                >
                  Añadir
                </button>
                <button
                  type="button"
                  onClick={() => setAdding(false)}
                  className="rounded-md px-3 py-1.5 text-[11.5px] text-neutral-500 dark:text-[#8B92A1]"
                >
                  Cancelar
                </button>
              </div>
            </form>
          ) : (
            <button
              type="button"
              onClick={() => setAdding(true)}
              className="w-full border-black/[0.08] px-3.5 py-3 text-left text-[12px] text-[#2F62D8] not-first:border-t dark:border-white/[0.08] dark:text-[#6E9BFF]"
            >
              Añadir un navegador
            </button>
          )}
        </Card>
      </Section>
    </>
  );
}
