import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { follow } from "../theme";
import Application from "./Application";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const PREFS = {
  schema_version: 1,
  theme: "system" as "system" | "light" | "dark",
  locale: "system" as "system" | "spanish" | "english",
  shortcut: "Alt+Shift+L" as string | null,
  notify_on_rule: true,
  picker_style: "classic" as "classic" | "sheet",
  looks_for_updates: true,
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

describe("on a Mac", () => {
  beforeEach(() => {
    invoke.mockReset();
    answers();
    Object.defineProperty(navigator, "platform", { value: "MacIntel", configurable: true });
  });
  afterEach(() => {
    Object.defineProperty(navigator, "platform", { value: "", configurable: true });
  });

  /// The menu bar icon is the only way back to the picker once the Dock icon is gone with it,
  /// so a Mac never offers to hide it.
  it("never offers to hide the menu bar icon", async () => {
    mount();
    await screen.findByRole("group", { name: "Tema" });
    expect(screen.queryByRole("switch", { name: "Ocultar el icono de la bandeja" })).toBeNull();
  });
});

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

  it("offers the tiles, and keeps the classic picker marked until they are chosen", async () => {
    mount();
    const picker = within(await screen.findByRole("group", { name: "Selector" }));
    expect(picker.getByRole("radio", { name: "Clásico" })).toBeChecked();
    await userEvent.click(picker.getByRole("radio", { name: "Mosaico" }));
    expect(invoke).toHaveBeenCalledWith("prefs_set", {
      prefs: { ...PREFS, picker_style: "sheet" },
    });
  });

  /// The resident asks the feed on its own; the switch is the one way to say not to, and it has
  /// to read back what is in force.
  it("lets the background look for releases be turned off", async () => {
    mount();
    const look = await screen.findByRole("switch", {
      name: "Buscar versiones nuevas en segundo plano",
    });
    expect(look).toBeChecked();
    await userEvent.click(look);
    expect(invoke).toHaveBeenCalledWith("prefs_set", {
      prefs: { ...PREFS, looks_for_updates: false },
    });
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

  it("surfaces the backend refusal rather than looking like it worked", async () => {
    answers(PREFS, "Alt+Shift+L", "the file could not be written");
    mount();
    await userEvent.click(await screen.findByRole("radio", { name: "Claro" }));
    expect(await screen.findByText(/could not be written/)).toBeInTheDocument();
  });

  it("turns the shortcut off when it is removed", async () => {
    mount();
    await userEvent.click(await screen.findByRole("button", { name: "Quitar" }));
    expect(invoke).toHaveBeenCalledWith("prefs_set", { prefs: { ...PREFS, shortcut: null } });
  });

  it("asks again after a change of combination, once the resident has claimed it", async () => {
    vi.useFakeTimers();
    try {
      answers(PREFS, "Alt+Shift+L");
      mount();
      const button = await vi.waitFor(() => screen.getByRole("button", { name: "Alt+Shift+L" }));

      answers({ ...PREFS, shortcut: "Control+Alt+B" }, null);
      fireEvent.click(button);
      fireEvent.keyDown(button, { code: "KeyB", ctrlKey: true, altKey: true });
      await vi.waitFor(() =>
        expect(invoke).toHaveBeenCalledWith("prefs_set", {
          prefs: { ...PREFS, shortcut: "Control+Alt+B" },
        }),
      );
      invoke.mockClear();

      await vi.advanceTimersByTimeAsync(1500);
      expect(invoke).toHaveBeenCalledWith("prefs_get");
      await vi.waitFor(() => expect(screen.getByText(/Otra aplicación ya usa/)).toBeTruthy());
    } finally {
      vi.useRealTimers();
    }
  });
});
