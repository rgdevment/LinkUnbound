import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { follow } from "../theme";
import Application from "./Application";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const PREFS = {
  schema_version: 1,
  theme: "system" as "system" | "light" | "dark",
  locale: "system" as "system" | "spanish" | "english",
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
  render(<Application startsWithSystem startupIsOurs onSystem={() => {}} onLanguage={() => {}} />);
}

describe("application settings", () => {
  beforeEach(() => {
    invoke.mockReset();
    answers();
  });

  /// The other tests assert the payload, not what the control ends up showing.
  it("marks the option that is actually in force", async () => {
    answers({ ...PREFS, theme: "dark", locale: "english" });
    mount();

    const theme = within(await screen.findByRole("group", { name: "Tema" }));
    expect(theme.getByRole("radio", { name: "Oscuro" })).toBeChecked();
    expect(theme.getByRole("radio", { name: "Automático" })).not.toBeChecked();

    const language = within(screen.getByRole("group", { name: "Idioma" }));
    expect(language.getByRole("radio", { name: "Inglés" })).toBeChecked();
    expect(language.getByRole("radio", { name: "Español" })).not.toBeChecked();
  });

  it("saves the theme as a choice of its own, not as whatever the system says", async () => {
    mount();
    await userEvent.click(await screen.findByRole("radio", { name: "Oscuro" }));
    expect(invoke).toHaveBeenCalledWith("prefs_set", {
      prefs: { ...PREFS, theme: "dark" },
    });
  });

  it("repaints the window as soon as the theme is chosen", async () => {
    invoke.mockImplementation((cmd: string) => {
      const prefs = { ...PREFS, theme: document.documentElement.dataset.picked ?? "system" };
      if (cmd === "prefs_set") document.documentElement.dataset.picked = "dark";
      return Promise.resolve({ prefs, shortcut_held: null });
    });
    vi.stubGlobal("matchMedia", () => ({ matches: false, addEventListener() {} }));
    follow();
    mount();
    await userEvent.click(await screen.findByRole("radio", { name: "Oscuro" }));
    await waitFor(() => expect(document.documentElement).toHaveClass("dark"));
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
