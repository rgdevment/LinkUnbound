import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Settings from "./Settings";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));
// Without this the window's update listener reaches for a Tauri runtime that is not there, and
// the suite reports every test passing while the run itself fails.
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => {}) }));

const BASE = {
  registered: true,
  is_default: true,
  associations: [
    { scheme: "http", held: true },
    { scheme: "https", held: true },
  ],
  starts_with_system: true,
  startup_is_ours: true,
  health: "fine",
  edge_installed: false,
};

const BUILD = {
  version: "2.0.0",
  license: "GPL-3.0-only",
  repository: "https://github.com/rgdevment/LinkUnbound",
  candidates: false,
  candidatesApply: true,
};

const PREFS = {
  prefs: {
    schema_version: 1,
    theme: "system",
    locale: "system",
    shortcut: "Alt+Shift+L",
    hide_tray: false,
    notify_on_rule: true,
    picker_style: "default",
  },
  shortcut_held: "Alt+Shift+L",
};

function answers(
  state: Record<string, unknown>,
  overrides: Record<string, () => Promise<unknown>> = {},
) {
  invoke.mockImplementation((cmd: string) => {
    if (cmd in overrides) return overrides[cmd]();
    if (cmd === "rules_list") return Promise.resolve([]);
    if (cmd === "prefs_get" || cmd === "prefs_set") return Promise.resolve(PREFS);
    if (cmd === "update_ready") return Promise.resolve(null);
    if (cmd === "about") return Promise.resolve(BUILD);
    return Promise.resolve(state);
  });
}

async function go(section: string) {
  await userEvent.click(await screen.findByRole("button", { name: section }));
}

describe("settings", () => {
  /// The sidebar's corner said "Versión {}" for two different facts: the one you run and the one
  /// waiting. With an offer in hand it showed the new number where the installed one had been.
  it("shows the version it is running when there is nothing newer", async () => {
    answers(BASE);
    render(<Settings />);
    expect(await screen.findByText("Versión 2.0.0")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /hay una versión nueva/ })).toBeNull();
  });

  it("offers a way in when a newer one is waiting, without claiming to be it", async () => {
    answers(BASE, {
      update_ready: () =>
        Promise.resolve({ version: "2.1.0", route: "download", package: null, installs: true }),
    });
    render(<Settings />);

    const go = await screen.findByRole("button", {
      name: "Acerca de · hay una versión nueva",
    });
    expect(go).toHaveTextContent("Ver la 2.1.0 disponible");
    expect(screen.queryByText("Versión 2.1.0")).toBeNull();

    await userEvent.click(go);
    expect(await screen.findByText("GPL-3.0-only")).toBeInTheDocument();
  });

  beforeEach(() => {
    invoke.mockReset();
    answers(BASE);
  });

  it("speaks whatever language the backend resolved", async () => {
    answers(BASE, { prefs_get: () => Promise.resolve({ ...PREFS, language: "en" }) });
    render(<Settings />);
    expect(await screen.findByRole("button", { name: "Links" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Maintenance" })).toBeInTheDocument();
  });

  it("changes language without a restart when the choice is made", async () => {
    answers(BASE, {
      prefs_get: () => Promise.resolve({ ...PREFS, language: "es" }),
      prefs_set: () =>
        Promise.resolve({
          ...PREFS,
          prefs: { ...PREFS.prefs, locale: "english" },
          language: "en",
        }),
    });
    render(<Settings />);
    await go("Aplicación");
    await userEvent.click(await screen.findByRole("radio", { name: "Inglés" }));
    expect(await screen.findByRole("button", { name: "Application" })).toBeInTheDocument();
  });

  /// A screen reader picks its voice from this attribute, so it has to follow
  /// the language actually being shown.
  it("tells the document which language it is speaking", async () => {
    answers(BASE, { prefs_get: () => Promise.resolve({ ...PREFS, language: "en" }) });
    render(<Settings />);
    await screen.findByRole("button", { name: "Links" });
    await waitFor(() => expect(document.documentElement.lang).toBe("en"));
  });

  it("names the binary the registration really points at", async () => {
    answers({
      ...BASE,
      health: "wrong_binary",
      registered_path: String.raw`"C:\Program Files\LinkUnbound\linkunbound-settings.exe" "%1"`,
    });
    render(<Settings />);
    expect(await screen.findByText(/no sabe abrir un enlace/)).toBeInTheDocument();
    expect(
      screen.getByText((text) => text.includes("linkunbound-settings.exe")),
    ).toBeInTheDocument();
  });

  it("does not hide a registration left behind by a build tree", async () => {
    answers({
      ...BASE,
      health: "build_tree",
      registered_path: String.raw`"D:\Code\LinkUnbound\target\debug\linkunbound-shell.exe" "%1"`,
    });
    render(<Settings />);
    expect(
      await screen.findByText((text) => text.includes(String.raw`target\debug`)),
    ).toBeInTheDocument();
  });

  it("says the links arrive here when they really do", async () => {
    render(<Settings />);
    expect(await screen.findByText(/recibe los enlaces/)).toBeInTheDocument();
  });

  /// The default browser is chosen in Windows, not here, so the answer changes while this window
  /// sits in the background. Asked once at startup, it went on saying the links do not arrive
  /// after the person had just made them arrive.
  it("asks again when it comes back from the Windows panel", async () => {
    answers({ ...BASE, is_default: false, associations: [] });
    render(<Settings />);
    expect(await screen.findByText(/todavía no envía/)).toBeInTheDocument();

    answers(BASE);
    window.dispatchEvent(new Event("focus"));

    expect(await screen.findByText(/recibe los enlaces/)).toBeInTheDocument();
  });

  it("opens the Windows panel when another browser holds the links", async () => {
    answers({ ...BASE, is_default: false });
    render(<Settings />);
    await userEvent.click(await screen.findByRole("button", { name: "Abrir Configuración" }));
    expect(invoke).toHaveBeenCalledWith("system_open_default_apps");
  });

  it("offers no Windows button once the links already arrive here", async () => {
    render(<Settings />);
    await screen.findByText(/recibe los enlaces/);
    expect(screen.queryByRole("button", { name: "Abrir Configuración" })).toBeNull();
  });

  /// Looking registered is not the same as working: the command can point at a
  /// path this executable no longer occupies.
  it("warns and offers to repair a registration pointing somewhere else", async () => {
    answers({ ...BASE, health: "stale" });
    render(<Settings />);
    expect(await screen.findByText(/no está recibiendo los enlaces/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Reparar" }));
    expect(invoke).toHaveBeenCalledWith("system_repair");
  });

  /// A build tree cannot own the registration at all, so there is nothing to
  /// repair and offering the button would lie.
  it("explains a build tree without offering a repair that cannot work", async () => {
    answers({ ...BASE, health: "build_tree" });
    render(<Settings />);
    expect(await screen.findByText(/recién compilada/)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Reparar" })).toBeNull();
  });

  it("says nothing about the handler when it is healthy", async () => {
    render(<Settings />);
    await screen.findByText(/recibe los enlaces/);
    expect(screen.queryByText(/no está recibiendo los enlaces/)).toBeNull();
  });

  /// The complaint that started the project: explaining it was portable even
  /// though intercepting the channel is not.
  it("explains the Edge channel only on a machine that has Edge", async () => {
    answers({ ...BASE, edge_installed: true });
    render(<Settings />);
    expect(await screen.findByText(/saltándose el navegador/)).toBeInTheDocument();
  });

  it("says nothing about Edge when Edge is not installed", async () => {
    render(<Settings />);
    await screen.findByText(/recibe los enlaces/);
    expect(screen.queryByText(/saltándose el navegador/)).toBeNull();
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

  describe("on a Mac", () => {
    beforeEach(() => {
      Object.defineProperty(navigator, "platform", { value: "MacIntel", configurable: true });
    });
    afterEach(() => {
      Object.defineProperty(navigator, "platform", { value: "", configurable: true });
    });

    it("offers to become the default, which macOS confirms with its own prompt", async () => {
      answers({ ...BASE, is_default: false });
      render(<Settings />);
      expect(await screen.findByText(/macOS todavía no envía/)).toBeInTheDocument();
      await userEvent.click(screen.getByRole("button", { name: "Establecer" }));
      expect(invoke).toHaveBeenCalledWith("system_set_registered", { enabled: true });
      expect(screen.queryByRole("switch", { name: "Ofrecerse como navegador" })).toBeNull();
    });

    it("sends to System Settings as the second way, never as the first", async () => {
      answers({ ...BASE, is_default: false });
      render(<Settings />);
      await userEvent.click(
        await screen.findByRole("button", { name: "Abrir Ajustes del Sistema" }),
      );
      expect(invoke).toHaveBeenCalledWith("system_open_default_apps");
    });

    it("offers nothing to press once the links already arrive here", async () => {
      render(<Settings />);
      await screen.findByText(/recibe los enlaces/);
      expect(screen.queryByRole("button", { name: "Establecer" })).toBeNull();
      expect(screen.queryByRole("button", { name: "Abrir Ajustes del Sistema" })).toBeNull();
    });

    it("says the prompt was declined in the reader's words", async () => {
      answers(
        { ...BASE, is_default: false },
        {
          system_set_registered: () => Promise.reject("defaultDeclined"),
        },
      );
      render(<Settings />);
      await userEvent.click(await screen.findByRole("button", { name: "Establecer" }));
      expect(await screen.findByText(/Rechazaste el cambio/)).toBeInTheDocument();
    });

    it("names the login items pane rather than a Windows one", async () => {
      answers({ ...BASE, starts_with_system: false, startup_is_ours: false });
      render(<Settings />);
      await go("Aplicación");
      expect(await screen.findByText(/Ítems de inicio/)).toBeInTheDocument();
    });
  });

  it("counts the associations it holds against the ones a browser is asked to carry", async () => {
    answers({
      ...BASE,
      is_default: false,
      associations: [
        { scheme: "http", held: true },
        { scheme: "https", held: false },
        { scheme: ".htm", held: false },
      ],
    });
    render(<Settings />);
    expect(await screen.findByText("1 de 3 asociaciones")).toBeInTheDocument();
    expect(screen.getByText(/todavía no envía/)).toBeInTheDocument();
  });

  it("says nothing about the links while the backend has not answered", async () => {
    answers(BASE, { system_state: () => new Promise(() => {}) });
    render(<Settings />);
    expect(await screen.findByRole("heading", { name: "Enlaces" })).toBeInTheDocument();
    expect(screen.getByText("0 de 0 asociaciones")).toBeInTheDocument();
    expect(screen.getByText(/todavía no envía/)).toBeInTheDocument();
    expect(screen.queryByText(/no está recibiendo/)).toBeNull();
  });

  it("keeps what it knew when an action answers with nothing", async () => {
    answers(
      { ...BASE, is_default: false },
      { system_open_default_apps: () => Promise.resolve(null) },
    );
    render(<Settings />);
    await userEvent.click(await screen.findByRole("button", { name: "Abrir Configuración" }));
    expect(screen.getByText(/todavía no envía/)).toBeInTheDocument();
    expect(screen.queryByText(/Algo salió mal/)).toBeNull();
  });

  it("clears the last complaint the moment another action is tried", async () => {
    let refuse = true;
    answers(BASE, {
      system_set_registered: () =>
        refuse ? Promise.reject("the registry refused the write") : Promise.resolve(BASE),
    });
    render(<Settings />);
    await userEvent.click(await screen.findByRole("switch", { name: "Ofrecerse como navegador" }));
    expect(await screen.findByText(/registry refused/)).toBeInTheDocument();
    refuse = false;
    await userEvent.click(screen.getByRole("switch", { name: "Ofrecerse como navegador" }));
    await waitFor(() => expect(screen.queryByText(/registry refused/)).toBeNull());
  });

  it("repairs a registration that points at another copy", async () => {
    answers({
      ...BASE,
      health: "stale",
      registered_path: String.raw`C:\Other\linkunbound-shell.exe`,
    });
    render(<Settings />);
    await userEvent.click(await screen.findByRole("button", { name: "Reparar" }));
    expect(invoke).toHaveBeenCalledWith("system_repair");
  });

  it("offers no repair for a resident that is not there", async () => {
    answers({ ...BASE, health: "no_resident" });
    render(<Settings />);
    expect(await screen.findByText(/Falta el programa/)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Reparar" })).toBeNull();
  });

  it("warns a copy running from the disk image, and offers no repair", async () => {
    answers({ ...BASE, health: "mounted" });
    render(<Settings />);
    expect(await screen.findByText(/imagen de disco/)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Reparar" })).toBeNull();
  });

  it("walks every section and marks the one it is on", async () => {
    answers(BASE, { browsers_list: () => Promise.resolve([]) });
    render(<Settings />);
    for (const [name, heading] of [
      ["Reglas", "Reglas"],
      ["Navegadores", "Navegadores"],
      ["Aplicación", "Aplicación"],
      ["Mantenimiento", "Mantenimiento"],
      ["Acerca de", "Acerca de"],
      ["Enlaces", "Enlaces"],
    ]) {
      await go(name);
      expect(await screen.findByRole("heading", { name: heading })).toBeInTheDocument();
      expect(screen.getByRole("button", { name })).toHaveAttribute("aria-current", "page");
      expect(screen.getAllByRole("button", { current: "page" })).toHaveLength(1);
    }
  });

  it("shows one section at a time", async () => {
    answers(BASE, { browsers_list: () => Promise.resolve([]) });
    render(<Settings />);
    expect(await screen.findByText("Navegador predeterminado")).toBeInTheDocument();
    expect(screen.queryByText(/Todavía no hay ninguna regla/)).toBeNull();
    await go("Reglas");
    expect(await screen.findByText(/Todavía no hay ninguna regla/)).toBeInTheDocument();
    expect(screen.queryByText("Navegador predeterminado")).toBeNull();
    await go("Acerca de");
    expect(await screen.findByText(/Estás en la última versión/)).toBeInTheDocument();
    expect(screen.queryByText(/Todavía no hay ninguna regla/)).toBeNull();
  });

  it("names no path when the registration names none", async () => {
    answers({ ...BASE, health: "stale", registered_path: null });
    render(<Settings />);
    expect(await screen.findByText(/apunta a otra copia/)).toBeInTheDocument();
    expect(screen.queryByText(/ejecutaría/)).toBeNull();
  });

  it("takes the startup toggle as off and ours while nothing is known", async () => {
    answers(BASE, { system_state: () => new Promise(() => {}) });
    render(<Settings />);
    await go("Aplicación");
    const toggle = await screen.findByRole("switch", { name: "Iniciar con el sistema" });
    expect(toggle).toBeEnabled();
    expect(toggle).toHaveAttribute("aria-checked", "false");
  });

  it("opens on the links section, not on a blank pane", async () => {
    render(<Settings />);
    expect(await screen.findByRole("heading", { name: "Enlaces" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Enlaces" })).toHaveAttribute("aria-current", "page");
  });
});
