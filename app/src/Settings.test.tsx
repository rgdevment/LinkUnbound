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
    if (cmd === "rules_list") return Promise.resolve([]);
    return Promise.resolve(state);
  });
}

async function go(section: string) {
  await userEvent.click(await screen.findByRole("button", { name: section }));
}

describe("settings", () => {
  beforeEach(() => {
    invoke.mockReset();
    answers(BASE);
  });

  it("says the links arrive here when they really do", async () => {
    render(<Settings />);
    expect(await screen.findByText(/recibe los enlaces/)).toBeInTheDocument();
  });

  it("points at Windows when another browser holds the links", async () => {
    answers({ ...BASE, is_default: false });
    render(<Settings />);
    expect(await screen.findByText(/Aplicaciones predeterminadas/)).toBeInTheDocument();
  });

  it("lets the registration be turned off", async () => {
    render(<Settings />);
    await userEvent.click(await screen.findByRole("switch", { name: "Ofrecerse como navegador" }));
    expect(invoke).toHaveBeenCalledWith("system_set_registered", { enabled: false });
  });

  /// Turning it back on from here would not work, so the switch has to say so
  /// rather than pretend the order was accepted.
  it("blocks startup when something outside disabled it, and explains why", async () => {
    answers({ ...BASE, starts_with_system: false, startup_is_ours: false });
    render(<Settings />);
    await go("Aplicación");
    expect(await screen.findByText(/Aplicaciones de inicio/)).toBeInTheDocument();
    expect(screen.getByRole("switch", { name: "Iniciar con el sistema" })).toBeDisabled();
  });

  it("shows the reason when the system refuses the change", async () => {
    answers(BASE, {
      system_set_registered: () => Promise.reject("the registry refused the write"),
    });
    render(<Settings />);
    await userEvent.click(await screen.findByRole("switch", { name: "Ofrecerse como navegador" }));
    expect(await screen.findByText(/registry refused/)).toBeInTheDocument();
  });

  it("opens on the links section, not on a blank pane", async () => {
    render(<Settings />);
    expect(await screen.findByRole("heading", { name: "Enlaces" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Enlaces" })).toHaveAttribute("aria-current", "page");
  });
});
