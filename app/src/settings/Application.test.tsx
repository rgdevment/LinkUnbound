import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Application from "./Application";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const PREFS = {
  schema_version: 1,
  theme: "system" as const,
  locale: "system" as const,
  shortcut: "Alt+Shift+L" as string | null,
  hide_tray: false,
  notify_on_rule: true,
};

function answers(prefs = PREFS, held: string | null = "Alt+Shift+L", fail?: string) {
  invoke.mockImplementation((cmd: string) => {
    if (cmd === "prefs_set" && fail) return Promise.reject(fail);
    return Promise.resolve({ prefs, shortcut_held: held });
  });
}

function mount() {
  render(<Application startsWithSystem startupIsOurs onSystem={() => {}} />);
}

describe("application settings", () => {
  beforeEach(() => {
    invoke.mockReset();
    answers();
  });

  it("saves the theme as a choice of its own, not as whatever the system says", async () => {
    mount();
    await userEvent.click(await screen.findByRole("radio", { name: "Oscuro" }));
    expect(invoke).toHaveBeenCalledWith("prefs_set", {
      prefs: { ...PREFS, theme: "dark" },
    });
  });

  it("captures a combination from the physical keys, not the printed ones", async () => {
    mount();
    const button = await screen.findByRole("button", { name: "Alt+Shift+L" });
    await userEvent.click(button);
    fireEvent.keyDown(button, { code: "KeyB", ctrlKey: true, altKey: true });
    expect(invoke).toHaveBeenCalledWith("prefs_set", {
      prefs: { ...PREFS, shortcut: "Control+Alt+B" },
    });
  });

  /// A lone letter would swallow every B the user types anywhere else.
  it("refuses a capture that carries no modifier", async () => {
    mount();
    const button = await screen.findByRole("button", { name: "Alt+Shift+L" });
    await userEvent.click(button);
    fireEvent.keyDown(button, { code: "KeyB" });
    expect(invoke).not.toHaveBeenCalledWith("prefs_set", expect.anything());
  });

  it("takes a function key on its own", async () => {
    mount();
    const button = await screen.findByRole("button", { name: "Alt+Shift+L" });
    await userEvent.click(button);
    fireEvent.keyDown(button, { code: "F9" });
    expect(invoke).toHaveBeenCalledWith("prefs_set", { prefs: { ...PREFS, shortcut: "F9" } });
  });

  it("says when another application already holds the combination", async () => {
    answers(PREFS, null);
    mount();
    expect(await screen.findByText(/Otra aplicación ya usa/)).toBeInTheDocument();
  });

  /// Hiding the tray with no shortcut left would strand the user, so the switch
  /// cannot even be reached until there is one.
  it("will not let the tray be hidden while there is no shortcut", async () => {
    answers({ ...PREFS, shortcut: null }, null);
    mount();
    expect(
      await screen.findByRole("switch", { name: "Ocultar el icono de la bandeja" }),
    ).toBeDisabled();
    expect(screen.getByText(/Necesitas un atajo/)).toBeInTheDocument();
  });

  it("surfaces the backend refusal rather than looking like it worked", async () => {
    answers(PREFS, "Alt+Shift+L", "con la bandeja oculta y sin atajo no habría forma de volver");
    mount();
    await userEvent.click(await screen.findByRole("radio", { name: "Claro" }));
    expect(await screen.findByText(/no habría forma de volver/)).toBeInTheDocument();
  });

  it("turns the shortcut off when it is removed", async () => {
    mount();
    await userEvent.click(await screen.findByRole("button", { name: "Quitar" }));
    expect(invoke).toHaveBeenCalledWith("prefs_set", { prefs: { ...PREFS, shortcut: null } });
  });
});
