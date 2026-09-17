import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Rules from "./Rules";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const SITE = {
  id: "site:github.com",
  browser_id: "chrome",
  profile_id: "Default",
  kind: "site" as const,
  covers: "github.com",
  browser: "Google Chrome",
  profile: "Tu Chrome",
  icon: "data:image/png;base64,iVBORw0KGgo=",
  private: false,
  source_app: null,
  resolved: true,
};

const LINK = {
  id: "url:https://mail.proton.me/inbox",
  browser_id: "firefox",
  profile_id: null,
  kind: "url" as const,
  covers: "https://mail.proton.me/inbox",
  browser: "Mozilla Firefox",
  profile: null,
  icon: null,
  private: true,
  source_app: null,
  resolved: true,
};

const FROM_TEAMS = {
  id: "any@teams",
  browser_id: "edge",
  profile_id: "Work",
  kind: "any" as const,
  covers: "",
  browser: "Microsoft Edge",
  profile: "Trabajo",
  icon: null,
  private: false,
  source_app: "teams",
  resolved: true,
};

const BROWSERS = [
  {
    id: "chrome",
    name: "Google Chrome",
    profile_names: [
      ["Default", "Tu Chrome"],
      ["Profile 2", "Trabajo"],
    ],
    private: true,
    hidden: false,
  },
  { id: "firefox", name: "Mozilla Firefox", profile_names: [], private: true, hidden: false },
  { id: "plain", name: "Plain Viewer", profile_names: [], private: false, hidden: false },
  {
    id: "edge",
    name: "Microsoft Edge",
    profile_names: [["Default", "Personal"]],
    private: true,
    hidden: true,
  },
];

function answers(list: unknown[], overrides: Record<string, () => Promise<unknown>> = {}) {
  invoke.mockImplementation((cmd: string) => {
    if (cmd in overrides) return overrides[cmd]();
    if (cmd === "browsers_list") return Promise.resolve(BROWSERS);
    return Promise.resolve(list);
  });
}

describe("rules", () => {
  beforeEach(() => {
    invoke.mockReset();
    answers([SITE, LINK, FROM_TEAMS]);
  });

  it("explains how rules are made instead of showing an empty box", async () => {
    answers([]);
    render(<Rules />);
    expect(await screen.findByText(/Todavía no hay ninguna regla/)).toBeInTheDocument();
    expect(screen.getByText(/Solo esta vez/)).toBeInTheDocument();
  });

  /// The empty screen used to send people to the picker and nowhere else; a rule for a site one
  /// has not visited yet is written here, and the same command the picker's choice would write.
  it("adds a rule from the empty screen, the way the picker would write it", async () => {
    answers([], { rules_add: () => Promise.resolve([SITE]) });
    render(<Rules />);
    await userEvent.click(await screen.findByRole("button", { name: "Añadir regla" }));

    await userEvent.selectOptions(screen.getByRole("combobox", { name: "Cuándo" }), "site");
    await userEvent.type(screen.getByRole("textbox", { name: "google.com" }), "github.com");
    await userEvent.selectOptions(
      screen.getByRole("combobox", { name: "Abrir en" }),
      "Google Chrome · Trabajo",
    );
    await userEvent.click(screen.getByRole("checkbox", { name: "En ventana privada" }));
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));

    expect(invoke).toHaveBeenCalledWith("rules_add", {
      kind: "site",
      value: "github.com",
      browserId: "chrome",
      profileId: "Profile 2",
      private: true,
    });
    expect(await screen.findByText("github.com")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Guardar" })).toBeNull();
  });

  it("offers the form above an existing list too", async () => {
    render(<Rules />);
    await userEvent.click(await screen.findByRole("button", { name: "Añadir regla" }));
    expect(screen.getByRole("combobox", { name: "Cuándo" })).toHaveDisplayValue("Todo el sitio");
    expect(screen.getByRole("textbox", { name: "google.com" })).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Cancelar" }));
    expect(screen.queryByRole("combobox", { name: "Cuándo" })).toBeNull();
  });

  /// The hint under the kind is the one thing that says what to type: it follows the kind.
  it("changes the hint with the kind, and sends no profile for a browser without one", async () => {
    answers([], { rules_add: () => Promise.resolve([LINK]) });
    render(<Rules />);
    await userEvent.click(await screen.findByRole("button", { name: "Añadir regla" }));
    await userEvent.selectOptions(screen.getByRole("combobox", { name: "Cuándo" }), "app");
    const box = screen.getByRole("textbox", { name: /Slack/ });
    await userEvent.type(box, "teams");
    await userEvent.selectOptions(
      screen.getByRole("combobox", { name: "Abrir en" }),
      "Mozilla Firefox",
    );
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    expect(invoke).toHaveBeenCalledWith("rules_add", {
      kind: "app",
      value: "teams",
      browserId: "firefox",
      profileId: null,
      private: false,
    });
  });

  /// Saved, such a rule would wear the badge and never fire: the box is greyed for a browser
  /// with no private window, and a tick made earlier is not sent along.
  it("cannot ask for a private window from a browser that has none", async () => {
    answers([], { rules_add: () => Promise.resolve([SITE]) });
    render(<Rules />);
    await userEvent.click(await screen.findByRole("button", { name: "Añadir regla" }));
    await userEvent.type(screen.getByRole("textbox", { name: "google.com" }), "github.com");
    const box = screen.getByRole("checkbox", { name: "En ventana privada" });
    await userEvent.click(box);
    expect(box).toBeChecked();
    await userEvent.selectOptions(
      screen.getByRole("combobox", { name: "Abrir en" }),
      "Plain Viewer",
    );
    expect(box).toBeDisabled();
    expect(box).not.toBeChecked();
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    expect(invoke).toHaveBeenCalledWith("rules_add", {
      kind: "site",
      value: "github.com",
      browserId: "plain",
      profileId: null,
      private: false,
    });
  });

  /// A refusal stays with the form, worded, so what was typed can be fixed rather than retyped.
  it("keeps the form and says why when the value is refused", async () => {
    answers([], { rules_add: () => Promise.reject("ruleValueNotHost") });
    render(<Rules />);
    await userEvent.click(await screen.findByRole("button", { name: "Añadir regla" }));
    await userEvent.type(screen.getByRole("textbox", { name: "google.com" }), "just words");
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    expect(await screen.findByText(/no es un dominio/)).toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: "google.com" })).toHaveValue("just words");
  });

  it("names what each rule covers and where it opens", async () => {
    render(<Rules />);
    expect(await screen.findByText("github.com")).toBeInTheDocument();
    expect(
      await screen.findByRole("combobox", { name: "Navegador para la regla de github.com" }),
    ).toHaveDisplayValue("Google Chrome · Tu Chrome");
  });

  /// A browser with profiles is one destination per profile, as in the picker.
  it("points a rule at another destination, profile and all", async () => {
    render(<Rules />);
    const picker = await screen.findByRole("combobox", {
      name: "Navegador para la regla de github.com",
    });
    await userEvent.selectOptions(picker, [
      within(picker).getByRole("option", { name: "Google Chrome · Trabajo" }),
    ]);
    expect(invoke).toHaveBeenCalledWith("rules_retarget", {
      id: "site:github.com",
      browserId: "chrome",
      profileId: "Profile 2",
    });
  });

  it("sends no profile for a browser that has none", async () => {
    render(<Rules />);
    const picker = await screen.findByRole("combobox", {
      name: "Navegador para la regla de github.com",
    });
    await userEvent.selectOptions(picker, [
      within(picker).getByRole("option", { name: "Mozilla Firefox" }),
    ]);
    expect(invoke).toHaveBeenCalledWith("rules_retarget", {
      id: "site:github.com",
      browserId: "firefox",
      profileId: null,
    });
  });

  /// Offering a hidden browser would put it back in the picker; not naming one
  /// a rule already points at shows somebody else's destination instead.
  it("names a destination it does not offer, rather than showing another one", async () => {
    answers([{ ...SITE, browser_id: "edge", profile_id: null, browser: "Microsoft Edge" }]);
    render(<Rules />);
    const picker = await screen.findByRole("combobox", {
      name: "Navegador para la regla de github.com",
    });
    expect(picker).toHaveDisplayValue("Microsoft Edge");
    expect(within(picker).queryByRole("option", { name: /Microsoft Edge · / })).toBeNull();
  });

  it("shows a rule bound to an origin by the app it came from", async () => {
    render(<Rules />);
    expect(await screen.findByText("teams")).toBeInTheDocument();
    expect(screen.getByText("Desde")).toBeInTheDocument();
  });

  it("marks the rules that open in a private window", async () => {
    render(<Rules />);
    expect(await screen.findByText("privada")).toBeInTheDocument();
  });

  it("asks before removing a rule, and says what is lost", async () => {
    render(<Rules />);
    await userEvent.click(
      await screen.findByRole("button", { name: "Eliminar la regla de github.com" }),
    );
    expect(invoke).not.toHaveBeenCalledWith("rules_remove", expect.anything());
    expect(await screen.findByText(/volverán a preguntarte/)).toBeInTheDocument();

    const ask = screen.getByRole("alertdialog", { name: "Eliminar la regla de github.com" });
    await userEvent.click(within(ask).getByRole("button", { name: "Eliminar" }));
    expect(invoke).toHaveBeenCalledWith("rules_remove", { id: "site:github.com" });
  });

  it("keeps the rule when the question is dismissed", async () => {
    render(<Rules />);
    await userEvent.click(
      await screen.findByRole("button", { name: "Eliminar la regla de github.com" }),
    );
    await userEvent.click(screen.getByRole("button", { name: "Cancelar" }));
    expect(invoke).not.toHaveBeenCalledWith("rules_remove", expect.anything());
  });

  /// Specificity decides, and two rules precise in the same way cannot both
  /// exist, so the order is not something the user can act on.
  it("offers no reordering, because the order decides nothing", async () => {
    render(<Rules />);
    await screen.findByText("github.com");
    expect(screen.queryByRole("button", { name: /Subir la regla/ })).toBeNull();
    expect(screen.queryByRole("button", { name: /Bajar la regla/ })).toBeNull();
  });

  it("says which rule wins instead of implying the order decides", async () => {
    render(<Rules />);
    expect(await screen.findByText(/regla más precisa/)).toBeInTheDocument();
  });

  it("surfaces a broken store instead of showing no rules at all", async () => {
    answers([], { rules_list: () => Promise.reject("rules.json could not be read") });
    render(<Rules />);
    expect(await screen.findByText(/could not be read/)).toBeInTheDocument();
  });

  /// Striking the name is what tells the user why the rule stopped working.
  it("strikes through a destination that is no longer installed", async () => {
    answers([{ ...SITE, resolved: false, browser: "brave", browser_id: "brave" }]);
    render(<Rules />);
    const picker = await screen.findByRole("combobox", {
      name: "Navegador para la regla de github.com",
    });
    expect(picker).toHaveClass("line-through");
    expect(within(picker).getByRole("option", { name: "brave" })).toBeInTheDocument();
  });
});
