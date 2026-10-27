import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Welcome from "./Welcome";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const ELSEWHERE = {
  registered: true,
  is_default: false,
  associations: [],
  starts_with_system: true,
  startup_is_ours: true,
  health: "not_registered",
  registered_path: null,
  edge_installed: false,
};
const HERE = { ...ELSEWHERE, is_default: true, health: "fine" };
const PREFS = { language: "es", prefs: { shortcut: "Alt+Shift+L" } };

function answers(overrides: Record<string, () => Promise<unknown>> = {}) {
  invoke.mockImplementation((cmd: string) => {
    if (cmd in overrides) return overrides[cmd]();
    if (cmd === "prefs_get") return Promise.resolve(PREFS);
    if (cmd === "system_state") return Promise.resolve(ELSEWHERE);
    if (cmd === "welcome_tried") return Promise.resolve(false);
    return Promise.resolve(null);
  });
}

async function press(name: string | RegExp) {
  await userEvent.click(await screen.findByRole("button", { name }));
}

describe("welcome", () => {
  beforeEach(() => {
    invoke.mockReset();
    answers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("greets, then asks for the links, then lets the picker be tried", async () => {
    render(<Welcome />);
    expect(
      await screen.findByRole("heading", { name: "Tú eliges dónde se abre cada enlace" }),
    ).toBeInTheDocument();
    await press("Comenzar");
    expect(
      screen.getByRole("heading", { name: "Que los enlaces lleguen a LinkUnbound" }),
    ).toBeInTheDocument();
    await press("Ahora no");
    expect(
      screen.getByRole("heading", { name: "Así se ve cada vez que abras un enlace" }),
    ).toBeInTheDocument();
    expect(screen.getByText(/se abrirá en tu navegador de siempre/)).toBeInTheDocument();
  });

  it("leaves out the links step for someone who already chose LinkUnbound", async () => {
    answers({ system_state: () => Promise.resolve(HERE) });
    render(<Welcome />);
    await press("Comenzar");
    expect(
      screen.getByRole("heading", { name: "Así se ve cada vez que abras un enlace" }),
    ).toBeInTheDocument();
    expect(screen.queryByText(/navegador de siempre/)).toBeNull();
  });

  it("closes, and so counts as seen, whether skipped or finished", async () => {
    render(<Welcome />);
    await press("Omitir");
    expect(invoke).toHaveBeenCalledWith("welcome_close");
  });

  /// Windows answers in a window of its own, so the step keeps asking and moves on by itself.
  it("waits for Windows and moves on once the links arrive", async () => {
    render(<Welcome />);
    await press("Comenzar");
    await press("Elegir LinkUnbound");
    expect(invoke).toHaveBeenCalledWith("system_open_default_apps");
    expect(await screen.findByText("Esperando a Windows…")).toBeInTheDocument();

    answers({ system_state: () => Promise.resolve(HERE) });
    expect(
      await screen.findByText("¡Listo! LinkUnbound ya recibe tus enlaces", {}, { timeout: 2000 }),
    ).toBeInTheDocument();
    expect(
      await screen.findByRole(
        "heading",
        { name: "Así se ve cada vez que abras un enlace" },
        { timeout: 3000 },
      ),
    ).toBeInTheDocument();
  });

  it("says in the reader's words why the choice did not go through", async () => {
    answers({
      system_state: () => Promise.resolve({ ...ELSEWHERE, registered: false }),
      system_set_registered: () => Promise.reject("packagedRegistration"),
    });
    render(<Welcome />);
    await press("Comenzar");
    await press("Elegir LinkUnbound");
    expect(await screen.findByRole("alert")).toHaveTextContent(/Microsoft Store/);
    expect(invoke).not.toHaveBeenCalledWith("system_open_default_apps");
  });

  /// The picker runs in another process; only what it leaves behind says it was used.
  it("waits for the picker before saying the test link worked", async () => {
    answers({ system_state: () => Promise.resolve(HERE) });
    render(<Welcome />);
    await press("Comenzar");
    await press(/rgdevment\.com/);
    expect(invoke).toHaveBeenCalledWith("welcome_try");
    expect(await screen.findByText(/Esperando a que elijas/)).toBeInTheDocument();
    expect(screen.queryByText(/Eso es/)).toBeNull();

    answers({
      system_state: () => Promise.resolve(HERE),
      welcome_tried: () => Promise.resolve(true),
    });
    expect(await screen.findByText(/Eso es/, {}, { timeout: 2000 })).toBeInTheDocument();
    expect(screen.getByText(/Alt\+Shift\+L/)).toBeInTheDocument();
    await press("Terminar");
    expect(invoke).toHaveBeenCalledWith("welcome_close");
  });

  it("still offers the links step when the system could not be asked", async () => {
    answers({ system_state: () => Promise.reject("unreadable") });
    render(<Welcome />);
    await press("Comenzar");
    expect(await screen.findByRole("button", { name: "Elegir LinkUnbound" })).toBeInTheDocument();
  });

  it("closes with Escape", async () => {
    render(<Welcome />);
    await screen.findByRole("button", { name: "Comenzar" });
    await act(async () => {
      await userEvent.keyboard("{Escape}");
    });
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("welcome_close"));
  });
});
