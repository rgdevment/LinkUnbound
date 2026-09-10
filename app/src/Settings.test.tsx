import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Settings from "./Settings";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const BASE = {
  registered: true,
  is_default: true,
  associations: [
    { scheme: "http", held: true },
    { scheme: "https", held: true },
  ],
  starts_with_system: true,
  startup_is_ours: true,
};

function answers(
  state: Record<string, unknown>,
  overrides: Record<string, () => Promise<unknown>> = {},
) {
  invoke.mockImplementation((cmd: string) => {
    if (cmd in overrides) return overrides[cmd]();
    return Promise.resolve(state);
  });
}

describe("ajustes", () => {
  beforeEach(() => {
    invoke.mockReset();
    answers(BASE);
  });

  it("dice que Windows abre los enlaces cuando de verdad los abre", async () => {
    render(<Settings />);
    expect(await screen.findByText(/Windows abre los enlaces/)).toBeInTheDocument();
  });

  it("explica que el cambio se hace en Windows cuando otro tiene los enlaces", async () => {
    answers({ ...BASE, is_default: false });
    render(<Settings />);
    expect(await screen.findByText(/Aplicaciones predeterminadas/)).toBeInTheDocument();
  });

  it("deja desactivar el registro", async () => {
    render(<Settings />);
    const toggles = await screen.findAllByRole("switch");
    await userEvent.click(toggles[0]);
    expect(invoke).toHaveBeenCalledWith("system_set_registered", { enabled: false });
  });

  /// Reactivarlo desde aquí no funcionaría: el interruptor tiene que decirlo en
  /// vez de fingir que la orden se aceptó.
  it("bloquea el arranque cuando lo desactivaron desde fuera, y lo explica", async () => {
    answers({ ...BASE, starts_with_system: false, startup_is_ours: false });
    render(<Settings />);
    expect(await screen.findByText(/Administrador de tareas/)).toBeInTheDocument();
    const toggles = screen.getAllByRole("switch");
    expect(toggles[1]).toBeDisabled();
  });

  it("muestra el motivo cuando el sistema rechaza el cambio", async () => {
    answers(BASE, {
      system_set_registered: () => Promise.reject("the registry refused the write"),
    });
    render(<Settings />);
    await userEvent.click((await screen.findAllByRole("switch"))[0]);
    expect(await screen.findByText(/registry refused/)).toBeInTheDocument();
  });
});
