import { fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Application from "./Application";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const PREFS = {
  schema_version: 1,
  locale: "system" as "system" | "spanish" | "english",
  shortcut: "Alt+Shift+L" as string | null,
  notify_on_rule: true,
  looks_for_updates: true,
};

function answers(
  prefs = PREFS,
  held: string | null = "Alt+Shift+L",
  fail?: string,
  installsHere = true,
) {
  invoke.mockImplementation((cmd: string) => {
    if (cmd === "prefs_set" && fail) return Promise.reject(fail);
    return Promise.resolve({ prefs, shortcut_held: held, installs_here: installsHere });
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
    await screen.findByRole("group", { name: "Idioma de la aplicación" });
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
    answers({ ...PREFS, locale: "english" });
    mount();

    const language = within(await screen.findByRole("group", { name: "Idioma de la aplicación" }));
    expect(language.getByRole("radio", { name: "Inglés" })).toBeChecked();
    expect(language.getByRole("radio", { name: "Español" })).not.toBeChecked();
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

  it("lets updates install on their own, and says so to the backend", async () => {
    mount();
    const automatic = await screen.findByRole("switch", {
      name: "Instalar las actualizaciones solas",
    });
    expect(automatic).not.toBeChecked();
    await userEvent.click(automatic);
    expect(invoke).toHaveBeenCalledWith("updates_automatically_set", { on: true });
  });

  it("does not offer to install on its own where the copy cannot replace itself", async () => {
    answers(PREFS, "Alt+Shift+L", undefined, false);
    mount();
    await screen.findByRole("switch", { name: "Buscar versiones nuevas en segundo plano" });
    expect(screen.queryByRole("switch", { name: "Instalar las actualizaciones solas" })).toBeNull();
  });

  it("disables installing on its own while the look for releases is off", async () => {
    answers({ ...PREFS, looks_for_updates: false });
    mount();
    expect(
      await screen.findByRole("switch", { name: "Instalar las actualizaciones solas" }),
    ).toBeDisabled();
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
    await userEvent.click(await screen.findByRole("radio", { name: "Inglés" }));
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

describe("what moved to the appearance page", () => {
  beforeEach(() => {
    invoke.mockReset();
    answers();
  });

  it("no longer offers the theme or the picker, but keeps the language", async () => {
    mount();
    await screen.findByRole("group", { name: "Idioma de la aplicación" });
    for (const gone of ["Tema", "Estilo", "Dónde aparece"]) {
      expect(screen.queryByRole("group", { name: gone })).toBeNull();
    }
    expect(screen.queryByRole("button", { name: "Vista previa" })).toBeNull();
  });
});
