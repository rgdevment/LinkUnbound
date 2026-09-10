import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";

type Association = { scheme: string; held: boolean };
type SystemState = {
  registered: boolean;
  is_default: boolean;
  associations: Association[];
  starts_with_system: boolean;
  startup_is_ours: boolean;
};

function Toggle({
  on,
  disabled,
  onChange,
}: {
  on: boolean;
  disabled?: boolean;
  onChange: (next: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      disabled={disabled}
      onClick={() => onChange(!on)}
      className={`h-[19px] w-[34px] shrink-0 rounded-full p-[2px] transition disabled:opacity-40 ${
        on ? "bg-[#89B4FA]" : "bg-[#45475A]"
      }`}
    >
      <span
        className={`block h-[15px] w-[15px] rounded-full transition ${
          on ? "ml-[15px] bg-[#1E1E2E]" : "bg-[#A6ADC8]"
        }`}
      />
    </button>
  );
}

export default function Settings() {
  const [state, setState] = useState<SystemState | null>(null);
  const [problem, setProblem] = useState<string | null>(null);

  const refresh = useCallback(() => {
    void invoke<SystemState>("system_state").then(setState);
  }, []);

  useEffect(refresh, [refresh]);

  const change = useCallback((command: string, enabled: boolean) => {
    setProblem(null);
    void invoke<SystemState>(command, { enabled })
      .then(setState)
      .catch((e: unknown) => setProblem(String(e)));
  }, []);

  const held = state?.associations.filter((a) => a.held).length ?? 0;
  const total = state?.associations.length ?? 0;

  return (
    <div className="min-h-full bg-[#1E1E2E] p-7 text-[#CDD6F4]">
      <h1 className="mb-1 text-[19px] font-semibold">Ajustes</h1>
      <p className="mb-6 text-[12.5px] text-[#A6ADC8]">
        LinkUnbound decide a qué navegador va cada enlace que abres.
      </p>

      <section className="mb-5 rounded-xl border border-[#313244] bg-[#181825] p-4">
        <h2 className="mb-3 text-[13px] font-semibold">Navegador predeterminado</h2>

        <div className="mb-3 flex items-center gap-3">
          <div className="flex-grow">
            <p className="text-[12.5px]">Ofrecerse como navegador</p>
            <p className="text-[11px] text-[#6C7086]">
              Hace que Windows liste LinkUnbound entre los navegadores disponibles.
            </p>
          </div>
          <Toggle
            on={state?.registered ?? false}
            onChange={(next) => change("system_set_registered", next)}
          />
        </div>

        <div className="rounded-lg bg-[#11111B] px-3 py-2.5 text-[11.5px]">
          {state?.is_default ? (
            <p className="text-[#A6E3A1]">
              Windows abre los enlaces con LinkUnbound. {held} de {total} asociaciones.
            </p>
          ) : (
            <p className="text-[#F9E2AF]">
              Otro programa tiene los enlaces. Windows solo permite cambiarlo desde su propia
              configuración: Aplicaciones predeterminadas → LinkUnbound.
            </p>
          )}
        </div>
      </section>

      <section className="mb-5 rounded-xl border border-[#313244] bg-[#181825] p-4">
        <h2 className="mb-3 text-[13px] font-semibold">Arranque</h2>
        <div className="flex items-center gap-3">
          <div className="flex-grow">
            <p className="text-[12.5px]">Iniciar con el sistema</p>
            <p className="text-[11px] text-[#6C7086]">
              {state?.startup_is_ours
                ? "Sin esto, el primer enlace de cada sesión tarda medio segundo en abrirse."
                : "Desactivado desde el Administrador de tareas; solo se puede reactivar desde ahí."}
            </p>
          </div>
          <Toggle
            on={state?.starts_with_system ?? false}
            disabled={!(state?.startup_is_ours ?? true)}
            onChange={(next) => change("system_set_startup", next)}
          />
        </div>
      </section>

      {problem && (
        <p className="rounded-lg bg-[#45293A] px-3 py-2.5 text-[11.5px] text-[#F38BA8]">
          {problem}
        </p>
      )}
    </div>
  );
}
