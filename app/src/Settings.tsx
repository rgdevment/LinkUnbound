import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import About from "./settings/About";
import Browsers from "./settings/Browsers";
import Maintenance from "./settings/Maintenance";
import { Card, Line, Section, Switch } from "./settings/parts";
import Rules from "./settings/Rules";

type Association = { scheme: string; held: boolean };
type SystemState = {
  registered: boolean;
  is_default: boolean;
  associations: Association[];
  starts_with_system: boolean;
  startup_is_ours: boolean;
};

type Page = "links" | "rules" | "browsers" | "app" | "care" | "about";

const PAGES: { id: Page; label: string; icon: React.ReactNode }[] = [
  {
    id: "links",
    label: "Enlaces",
    icon: (
      <>
        <path d="M10 13a5 5 0 0 0 7.5.5l3-3a5 5 0 0 0-7-7l-1.7 1.7" />
        <path d="M14 11a5 5 0 0 0-7.5-.5l-3 3a5 5 0 0 0 7 7l1.7-1.7" />
      </>
    ),
  },
  { id: "rules", label: "Reglas", icon: <path d="M4 6h16M4 12h11M4 18h7" /> },
  {
    id: "browsers",
    label: "Navegadores",
    icon: (
      <>
        <circle cx="12" cy="12" r="9" />
        <path d="M3 12h18M12 3a15 15 0 0 1 0 18M12 3a15 15 0 0 0 0 18" />
      </>
    ),
  },
  {
    id: "app",
    label: "Aplicación",
    icon: (
      <>
        <circle cx="12" cy="12" r="3" />
        <path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9l2.2 2.2M16.9 16.9l2.2 2.2M19.1 4.9l-2.2 2.2M7.1 16.9l-2.2 2.2" />
      </>
    ),
  },
  {
    id: "care",
    label: "Mantenimiento",
    icon: (
      <>
        <path d="M14.7 6.3a4 4 0 0 0 5 5l-9 9a2.8 2.8 0 0 1-4-4z" />
        <path d="M17.5 3.5 21 7" />
      </>
    ),
  },
  {
    id: "about",
    label: "Acerca de",
    icon: (
      <>
        <circle cx="12" cy="12" r="9" />
        <path d="M12 16v-4M12 8h.01" />
      </>
    ),
  },
];

function Links({
  state,
  change,
}: {
  state: SystemState | null;
  change: (command: string, enabled: boolean) => void;
}) {
  const held = state?.associations.filter((a) => a.held).length ?? 0;
  const total = state?.associations.length ?? 0;
  const ok = state?.is_default ?? false;

  return (
    <>
      <Section title="Navegador predeterminado">
        <div className="flex items-center gap-2 rounded-lg border border-black/[0.08] bg-black/[0.02] px-3 py-2.5 text-[11.5px] dark:border-white/[0.08] dark:bg-white/[0.02]">
          <span
            className={`h-[7px] w-[7px] shrink-0 rounded-full ${ok ? "bg-[#1E7A52] dark:bg-[#4CC38A]" : "bg-[#A85B14] dark:bg-[#E9A05C]"}`}
          />
          <span className="flex-1">
            {ok
              ? "LinkUnbound recibe los enlaces de este equipo"
              : "Windows todavía no envía los enlaces aquí"}
          </span>
          <span className="text-neutral-500 dark:text-[#8B92A1]">
            {held} de {total} asociaciones
          </span>
        </div>
        <Card>
          <Line
            title="Ofrecerse como navegador"
            note="Aparece en la lista de Windows para que puedas elegirlo"
          >
            <Switch
              on={state?.registered ?? false}
              label="Ofrecerse como navegador"
              onChange={(next) => change("system_set_registered", next)}
            />
          </Line>
        </Card>
        {!ok && (
          <p className="text-[11px] text-neutral-500 dark:text-[#8B92A1]">
            Solo tú puedes fijar el predeterminado, desde Configuración de Windows &gt; Aplicaciones
            predeterminadas.
          </p>
        )}
      </Section>
    </>
  );
}

function AppPage({
  state,
  change,
}: {
  state: SystemState | null;
  change: (command: string, enabled: boolean) => void;
}) {
  return (
    <Section title="Inicio">
      <Card>
        <Line
          title="Iniciar con el sistema"
          note={
            state?.startup_is_ours === false
              ? "Gestionado desde Configuración de Windows > Aplicaciones de inicio"
              : "El primer enlace de cada sesión se abre al instante"
          }
        >
          <Switch
            on={state?.starts_with_system ?? false}
            disabled={state?.startup_is_ours === false}
            label="Iniciar con el sistema"
            onChange={(next) => change("system_set_startup", next)}
          />
        </Line>
      </Card>
    </Section>
  );
}

export default function Settings() {
  const [page, setPage] = useState<Page>("links");
  const [state, setState] = useState<SystemState | null>(null);
  const [problem, setProblem] = useState<string | null>(null);

  useEffect(() => {
    void invoke<SystemState>("system_state").then(setState).catch(noop);
  }, []);

  const change = useCallback((command: string, enabled: boolean) => {
    setProblem(null);
    void invoke<SystemState>(command, { enabled })
      .then(setState)
      .catch((e: unknown) => setProblem(String(e)));
  }, []);

  return (
    <div className="flex h-full bg-white text-[#16181D] dark:bg-[#191A1F] dark:text-[#F0F1F4]">
      <nav
        aria-label="Secciones"
        className="flex w-44 shrink-0 flex-col gap-0.5 border-r border-black/[0.08] bg-black/[0.02] p-2 dark:border-white/[0.08] dark:bg-white/[0.02]"
      >
        {PAGES.map((p) => (
          <button
            key={p.id}
            type="button"
            aria-current={p.id === page ? "page" : undefined}
            onClick={() => setPage(p.id)}
            className={`flex items-center gap-2.5 rounded-md px-2.5 py-[7px] text-left text-[12.5px] transition ${
              p.id === page
                ? "bg-[#2F62D8]/[0.11] font-semibold text-[#2F62D8] dark:bg-[#6E9BFF]/[0.16] dark:text-[#6E9BFF]"
                : "text-neutral-500 hover:bg-black/[0.04] hover:text-inherit dark:text-[#8B92A1] dark:hover:bg-white/[0.06]"
            }`}
          >
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.8"
              strokeLinecap="round"
              className="h-[15px] w-[15px] shrink-0"
              aria-hidden="true"
            >
              {p.icon}
            </svg>
            {p.label}
          </button>
        ))}
        <span className="mt-auto px-2.5 py-2 text-[10.5px] text-neutral-400 dark:text-[#646B7C]">
          Versión 2.0.0
        </span>
      </nav>

      <main className="flex min-w-0 flex-1 flex-col gap-4 overflow-y-auto p-5">
        <h1 className="text-[15px] font-semibold">{PAGES.find((p) => p.id === page)?.label}</h1>
        {problem && (
          <p className="rounded-md bg-[#C0362F]/10 px-3 py-2 text-[11.5px] text-[#C0362F] dark:bg-[#FF8A85]/10 dark:text-[#FF8A85]">
            {problem}
          </p>
        )}
        {page === "links" && <Links state={state} change={change} />}
        {page === "rules" && <Rules />}
        {page === "browsers" && <Browsers />}
        {page === "app" && <AppPage state={state} change={change} />}
        {page === "care" && <Maintenance />}
        {page === "about" && <About />}
      </main>
    </div>
  );
}

function noop() {}
