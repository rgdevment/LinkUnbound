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
  health: "fine",
  edge_installed: false,
};

const PREFS = {
  prefs: {
    schema_version: 1,
    theme: "system",
    locale: "system",
    shortcut: "Alt+Shift+L",
    hide_tray: false,
    notify_on_rule: true,
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
    expect(document.documentElement.lang).toBe("en");
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

  it("opens the Windows panel when another browser holds the links", async () => {
    answers({ ...BASE, is_default: false });
    render(<Settings />);
    await userEvent.click(await screen.findByRole("button", { name: "Abrir Windows" }));
    expect(invoke).toHaveBeenCalledWith("system_open_default_apps");
  });

  it("offers no Windows button once the links already arrive here", async () => {
    render(<Settings />);
    await screen.findByText(/recibe los enlaces/);
    expect(screen.queryByRole("button", { name: "Abrir Windows" })).toBeNull();
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

  it("opens on the links section, not on a blank pane", async () => {
    render(<Settings />);
    expect(await screen.findByRole("heading", { name: "Enlaces" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Enlaces" })).toHaveAttribute("aria-current", "page");
  });
});
