import { act, fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
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
  profile_gone: false,
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
  profile_gone: false,
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
  profile_gone: false,
};

const GONE = {
  id: "site:gitlab.com",
  browser_id: "brave",
  profile_id: null,
  kind: "site" as const,
  covers: "gitlab.com",
  browser: "brave",
  profile: null,
  icon: null,
  private: false,
  source_app: null,
  resolved: false,
  profile_gone: false,
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
    icon: null,
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

const APP_BOX = "Busca una aplicación o escribe su nombre";

type Answer = () => Promise<unknown>;

function answers(list: unknown[], overrides: Record<string, Answer> = {}) {
  invoke.mockImplementation((cmd: string) => {
    if (cmd in overrides) return overrides[cmd]();
    if (cmd === "browsers_list") return Promise.resolve(BROWSERS);
    if (cmd === "apps_seen") return Promise.resolve([]);
    return Promise.resolve(list);
  });
}

const calls = (command: string) => invoke.mock.calls.filter(([c]) => c === command);

/// The list of rules, not the one inside the probe panel.
function theList() {
  const found = screen.getAllByRole("list").find((l) => !l.getAttribute("aria-label"));
  if (!found) throw new Error("no list of rules on screen");
  return found;
}

async function destination(of: string) {
  return await screen.findByRole("button", { name: `Navegador para la regla de ${of}` });
}

describe("rules", () => {
  beforeEach(() => {
    invoke.mockReset();
    answers([SITE, LINK, FROM_TEAMS, GONE]);
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

    await userEvent.click(screen.getByRole("radio", { name: "Todo el sitio" }));
    await userEvent.type(screen.getByRole("textbox", { name: "google.com" }), "github.com");
    await userEvent.click(screen.getByRole("button", { name: "Abrir en" }));
    await userEvent.click(screen.getByRole("option", { name: /Google Chrome · Trabajo/ }));
    await userEvent.click(screen.getByRole("checkbox", { name: "En ventana privada" }));
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));

    expect(invoke).toHaveBeenCalledWith("rules_add", {
      kind: "site",
      value: "github.com",
      browserId: "chrome",
      profileId: "Profile 2",
      private: true,
    });
    expect(await screen.findByTitle("github.com")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Guardar" })).toBeNull();
  });

  it("offers the form above an existing list too", async () => {
    render(<Rules />);
    await userEvent.click(await screen.findByRole("button", { name: "Añadir regla" }));
    expect(screen.getByRole("radio", { name: "Todo el sitio" })).toBeChecked();
    expect(screen.getByRole("textbox", { name: "google.com" })).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Cancelar" }));
    expect(screen.queryByRole("radio", { name: "Todo el sitio" })).toBeNull();
  });

  /// The hint under the kind is the one thing that says what to type: it follows the kind.
  it("changes the hint with the kind, and sends no profile for a browser without one", async () => {
    answers([], { rules_add: () => Promise.resolve([LINK]) });
    render(<Rules />);
    await userEvent.click(await screen.findByRole("button", { name: "Añadir regla" }));
    await userEvent.click(screen.getByRole("radio", { name: "Los enlaces de una aplicación" }));
    const box = screen.getByRole("combobox", { name: APP_BOX });
    await userEvent.type(box, "teams");
    await userEvent.click(screen.getByRole("button", { name: "Abrir en" }));
    await userEvent.click(screen.getByRole("option", { name: /Mozilla Firefox/ }));
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
    await userEvent.click(screen.getByRole("button", { name: "Abrir en" }));
    await userEvent.click(screen.getByRole("option", { name: /Plain Viewer/ }));
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
    expect(await screen.findByTitle("github.com")).toBeInTheDocument();
    const where = await destination("github.com");
    expect(where).toHaveAttribute("aria-haspopup", "listbox");
    expect(where).toHaveTextContent("Google Chrome");
    expect(where).toHaveTextContent("Tu Chrome");
  });

  it("keeps the full value of a long address in its title", async () => {
    render(<Rules />);
    expect(await screen.findByTitle("https://mail.proton.me/inbox")).toBeInTheDocument();
  });

  /// A browser with profiles is one destination per profile, as in the picker.
  it("points a rule at another destination, profile and all", async () => {
    render(<Rules />);
    await userEvent.click(await destination("github.com"));
    const options = within(screen.getByRole("listbox")).getAllByRole("option");
    expect(options.map((o) => o.textContent)).toEqual([
      expect.stringContaining("Google Chrome · Tu Chrome"),
      expect.stringContaining("Google Chrome · Trabajo"),
      expect.stringContaining("Mozilla Firefox"),
      expect.stringContaining("Plain Viewer"),
    ]);
    await userEvent.click(options[1]);
    expect(invoke).toHaveBeenCalledWith("rules_retarget", {
      id: "site:github.com",
      browserId: "chrome",
      profileId: "Profile 2",
    });
    expect(screen.queryByRole("listbox")).toBeNull();
  });

  it("sends no profile for a browser that has none", async () => {
    render(<Rules />);
    await userEvent.click(await destination("github.com"));
    await userEvent.click(screen.getByRole("option", { name: /Mozilla Firefox/ }));
    expect(invoke).toHaveBeenCalledWith("rules_retarget", {
      id: "site:github.com",
      browserId: "firefox",
      profileId: null,
    });
  });

  it("points a rule at another destination from the keyboard alone", async () => {
    render(<Rules />);
    await userEvent.click(await destination("github.com"));
    await userEvent.keyboard("{ArrowDown}{Enter}");
    expect(invoke).toHaveBeenCalledWith("rules_retarget", {
      id: "site:github.com",
      browserId: "chrome",
      profileId: "Profile 2",
    });
    expect(screen.queryByRole("listbox")).toBeNull();
    expect(await destination("github.com")).toHaveFocus();
  });

  it("closes the list with Escape and changes nothing", async () => {
    render(<Rules />);
    await userEvent.click(await destination("github.com"));
    await userEvent.keyboard("{ArrowDown}{Escape}");
    expect(screen.queryByRole("listbox")).toBeNull();
    expect(calls("rules_retarget")).toHaveLength(0);
    expect(await destination("github.com")).toHaveFocus();
  });

  /// Offering a hidden browser would put it back in the picker; not naming one
  /// a rule already points at shows somebody else's destination instead.
  it("names a destination it does not offer, rather than showing another one", async () => {
    answers([{ ...SITE, browser_id: "edge", profile_id: null, browser: "Microsoft Edge" }]);
    render(<Rules />);
    const picker = await destination("github.com");
    expect(picker).toHaveTextContent("Microsoft Edge");
    await userEvent.click(picker);
    expect(screen.queryByRole("option", { name: /Microsoft Edge/ })).toBeNull();
  });

  it("shows a rule bound to an origin by the app it came from", async () => {
    render(<Rules />);
    expect(await screen.findByTitle("teams")).toBeInTheDocument();
    const row = screen.getByTitle("teams").closest("li") as HTMLElement;
    expect(within(row).getByText("Desde")).toBeInTheDocument();
  });

  it("marks the rules that open in a private window", async () => {
    render(<Rules />);
    expect(await screen.findByText("privada")).toBeInTheDocument();
  });

  /// In a private rule a browser with no private window would turn the rule into one that never
  /// fires: it is shown, greyed, and cannot be chosen.
  it("greys the browsers without a private window in a private rule's picker", async () => {
    render(<Rules />);
    await userEvent.click(await destination("https://mail.proton.me/inbox"));
    const plain = screen.getByRole("option", { name: /Plain Viewer/ });
    expect(plain).toHaveAttribute("aria-disabled", "true");
    expect(plain).toHaveTextContent("sin ventana privada");
    await userEvent.click(plain);
    expect(calls("rules_retarget")).toHaveLength(0);
    expect(screen.getByRole("listbox")).toBeInTheDocument();
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

  it("keeps the edit and delete buttons in sight on every row", async () => {
    render(<Rules />);
    await screen.findByTitle("github.com");
    for (const of of ["github.com", "https://mail.proton.me/inbox", "teams", "gitlab.com"]) {
      expect(screen.getByRole("button", { name: `Editar la regla de ${of}` })).toBeVisible();
      expect(screen.getByRole("button", { name: `Eliminar la regla de ${of}` })).toBeVisible();
    }
  });

  /// Specificity decides, and two rules precise in the same way cannot both
  /// exist, so the order is not something the user can act on.
  it("offers no reordering, because the order decides nothing", async () => {
    render(<Rules />);
    await screen.findByTitle("github.com");
    expect(screen.queryByRole("button", { name: /Subir la regla/ })).toBeNull();
    expect(screen.queryByRole("button", { name: /Bajar la regla/ })).toBeNull();
  });

  it("says which rule wins instead of implying the order decides", async () => {
    render(<Rules />);
    expect(await screen.findByText(/Gana la más específica/)).toBeInTheDocument();
    expect(screen.getByText(/solo desempata/)).toBeInTheDocument();
  });

  it("surfaces a broken store instead of showing no rules at all", async () => {
    answers([], { rules_list: () => Promise.reject("rules.json could not be read") });
    render(<Rules />);
    expect(await screen.findByText(/could not be read/)).toBeInTheDocument();
  });

  describe("search and filters", () => {
    it("titles the list with how many of all the rules the search leaves", async () => {
      render(<Rules />);
      expect(await screen.findByRole("heading", { name: "4 reglas" })).toBeInTheDocument();
      await userEvent.type(screen.getByRole("searchbox"), "git");
      expect(screen.getByRole("heading", { name: "2 de 4 reglas" })).toBeInTheDocument();
    });

    it("marks where the search found a match, whatever the case", async () => {
      const { container } = render(<Rules />);
      await userEvent.type(await screen.findByRole("searchbox"), "GIT");
      const marks = [...container.querySelectorAll("mark")];
      expect(marks.map((m) => m.textContent)).toEqual(["git", "git"]);
      expect(theList()).not.toHaveTextContent("proton");
    });

    it.each([
      ["teams", "the app", "teams"],
      ["firefox", "the browser", "https://mail.proton.me/inbox"],
      ["tu chrome", "the profile", "github.com"],
    ])("finds a rule by %s, %s", async (said, _where, title) => {
      render(<Rules />);
      await userEvent.type(await screen.findByRole("searchbox"), said);
      expect(screen.getByRole("heading", { name: "1 de 4 reglas" })).toBeInTheDocument();
      expect(within(theList()).getByTitle(title)).toBeInTheDocument();
    });

    it("says nothing matches, and the empty search brings every rule back", async () => {
      render(<Rules />);
      await userEvent.type(await screen.findByRole("searchbox"), "zzz");
      expect(screen.getByText("Ninguna regla coincide con la búsqueda")).toBeInTheDocument();
      await userEvent.click(screen.getByRole("button", { name: "Borrar la búsqueda" }));
      expect(screen.getByRole("searchbox")).toHaveValue("");
      expect(within(theList()).getAllByRole("listitem")).toHaveLength(4);
    });

    it("counts each kind of rule and keeps only that kind when asked", async () => {
      render(<Rules />);
      const chip = async (name: RegExp) => await screen.findByRole("button", { name });
      expect(await chip(/^Todas/)).toHaveTextContent("Todas4");
      expect(await chip(/^Desde/)).toHaveTextContent("Desde1");
      expect(await chip(/^Esta URL/)).toHaveTextContent("Esta URL1");
      expect(await chip(/^Subdominio/)).toHaveTextContent("Subdominio0");
      expect(await chip(/^Sitio/)).toHaveTextContent("Sitio2");

      await userEvent.click(await chip(/^Sitio/));
      expect(await chip(/^Sitio/)).toHaveAttribute("aria-pressed", "true");
      expect(within(theList()).getAllByRole("listitem")).toHaveLength(2);
      expect(screen.getByRole("heading", { name: "2 de 4 reglas" })).toBeInTheDocument();
    });

    it("counts the kinds among what the search leaves", async () => {
      render(<Rules />);
      await userEvent.type(await screen.findByRole("searchbox"), "git");
      expect(screen.getByRole("button", { name: /^Todas/ })).toHaveTextContent("Todas2");
      expect(screen.getByRole("button", { name: /^Desde/ })).toHaveTextContent("Desde0");
      expect(screen.getByRole("button", { name: /^Sitio/ })).toHaveTextContent("Sitio2");
    });

    it("offers the filter for rules without a browser only when one is broken", async () => {
      answers([SITE, LINK]);
      render(<Rules />);
      await screen.findByTitle("github.com");
      expect(screen.queryByRole("button", { name: /^Sin navegador/ })).toBeNull();
    });

    it("colours the filter chips by token: ink-2 when idle, caution for the broken one", async () => {
      render(<Rules />);
      const broken = await screen.findByRole("button", { name: /^Sin navegador/ });
      expect(broken).toHaveClass("text-caution");
      expect(await screen.findByRole("button", { name: /^Sitio/ })).toHaveClass("text-ink-2");
      await userEvent.click(broken);
      expect(broken).toHaveClass("bg-caution/[0.12]");
    });

    it("gives every Rules control a hover and a focus ring", async () => {
      render(<Rules />);
      const add = await screen.findByRole("button", { name: "Añadir regla" });
      const probe = screen.getByRole("button", { name: /Probar un enlace/ });
      const chip = await screen.findByRole("button", { name: /^Sitio/ });
      const broken = screen.getByRole("button", { name: /^Sin navegador/ });
      for (const control of [add, probe, chip, broken]) {
        expect(control.className).toContain("hover:");
        expect(control.className).toContain("focus-visible:outline");
      }
    });

    it("keeps only the rules whose browser is gone when asked", async () => {
      render(<Rules />);
      const chip = await screen.findByRole("button", { name: /^Sin navegador/ });
      expect(chip).toHaveTextContent("Sin navegador1");
      await userEvent.click(chip);
      const rows = within(theList()).getAllByRole("listitem");
      expect(rows).toHaveLength(1);
      expect(within(rows[0]).getByTitle("gitlab.com")).toBeInTheDocument();
    });
  });

  describe("a rule whose browser is gone", () => {
    it("drops the filter for broken rules once the last one is fixed", async () => {
      answers([SITE, GONE], {
        rules_retarget: () => Promise.resolve([SITE, { ...GONE, resolved: true }]),
      });
      render(<Rules />);
      await userEvent.click(await screen.findByRole("button", { name: /^Sin navegador/ }));
      const row = screen.getByTitle("gitlab.com").closest("li") as HTMLElement;
      await userEvent.click(within(row).getByRole("button", { name: "Elegir otro navegador" }));
      await userEvent.click(screen.getByRole("option", { name: /Mozilla Firefox/ }));
      expect(await screen.findByTitle("github.com")).toBeInTheDocument();
      expect(screen.getByRole("button", { name: /^Todas/ })).toHaveAttribute(
        "aria-pressed",
        "true",
      );
      expect(screen.queryByText("Ninguna regla coincide con la búsqueda")).toBeNull();
    });

    it("says so in its row, and names the browser it lost", async () => {
      render(<Rules />);
      const row = (await screen.findByTitle("gitlab.com")).closest("li") as HTMLElement;
      expect(within(row).getByText(/brave ya no está instalado/)).toBeInTheDocument();
      expect(within(row).getByText(/el selector vuelve a preguntar/)).toBeInTheDocument();
    });

    it("opens its own destination list from «Elegir otro navegador»", async () => {
      render(<Rules />);
      const row = (await screen.findByTitle("gitlab.com")).closest("li") as HTMLElement;
      await userEvent.click(within(row).getByRole("button", { name: "Elegir otro navegador" }));

      const list = within(row).getByRole("listbox");
      await userEvent.click(within(list).getByRole("option", { name: /Mozilla Firefox/ }));
      expect(invoke).toHaveBeenCalledWith("rules_retarget", {
        id: "site:gitlab.com",
        browserId: "firefox",
        profileId: null,
      });
    });

    it("shows the rows that resolve without the note", async () => {
      render(<Rules />);
      await screen.findByTitle("github.com");
      expect(screen.getAllByText(/ya no está instalado/)).toHaveLength(1);
      expect(screen.getAllByRole("button", { name: "Elegir otro navegador" })).toHaveLength(1);
    });
  });

  describe("editing a rule in its place", () => {
    async function edit(of: string) {
      render(<Rules />);
      await userEvent.click(
        await screen.findByRole("button", { name: `Editar la regla de ${of}` }),
      );
      return await screen.findByRole("form", { name: `Editar la regla de ${of}` });
    }

    it("opens the form with what the rule says", async () => {
      const form = await edit("github.com");
      expect(within(form).getByRole("radio", { name: "Todo el sitio" })).toBeChecked();
      expect(within(form).getByRole("textbox", { name: "google.com" })).toHaveValue("github.com");
      expect(within(form).getByRole("button", { name: "Abrir en" })).toHaveTextContent(
        "Google Chrome · Tu Chrome",
      );
      expect(within(form).getByText(/conserva su lugar/)).toBeInTheDocument();
    });

    it("starts an app rule on the app kind, with the app typed", async () => {
      const form = await edit("teams");
      expect(
        within(form).getByRole("radio", { name: "Los enlaces de una aplicación" }),
      ).toBeChecked();
      expect(within(form).getByRole("combobox", { name: APP_BOX })).toHaveValue("teams");
    });

    it("sends the new rule with the id of the one it rewrites", async () => {
      const form = await edit("github.com");
      const value = within(form).getByRole("textbox", { name: "google.com" });
      await userEvent.clear(value);
      await userEvent.type(value, "example.org");
      await userEvent.click(within(form).getByRole("radio", { name: "Este dominio" }));
      await userEvent.click(within(form).getByRole("button", { name: "Abrir en" }));
      await userEvent.click(within(form).getByRole("option", { name: /Mozilla Firefox/ }));
      await userEvent.click(within(form).getByRole("checkbox", { name: "En ventana privada" }));
      await userEvent.click(within(form).getByRole("button", { name: "Guardar" }));

      expect(invoke).toHaveBeenCalledWith("rules_edit", {
        id: "site:github.com",
        kind: "host",
        value: "example.org",
        browserId: "firefox",
        profileId: null,
        private: true,
      });
      expect(screen.queryByRole("form", { name: /Editar la regla/ })).toBeNull();
    });

    it("closes with the list the command answered", async () => {
      invoke.mockImplementation((cmd: string) => {
        if (cmd === "browsers_list") return Promise.resolve(BROWSERS);
        if (cmd === "rules_edit") return Promise.resolve([{ ...SITE, covers: "example.org" }]);
        return Promise.resolve([SITE]);
      });
      render(<Rules />);
      await userEvent.click(
        await screen.findByRole("button", { name: "Editar la regla de github.com" }),
      );
      await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
      expect(await screen.findByTitle("example.org")).toBeInTheDocument();
    });

    /// What was typed stays, with the reason under it, so it can be mended instead of retyped.
    it("keeps the form open and the refusal under it", async () => {
      invoke.mockImplementation((cmd: string) => {
        if (cmd === "browsers_list") return Promise.resolve(BROWSERS);
        if (cmd === "rules_edit") return Promise.reject("ruleExists");
        return Promise.resolve([SITE, LINK]);
      });
      render(<Rules />);
      await userEvent.click(
        await screen.findByRole("button", { name: "Editar la regla de github.com" }),
      );
      const form = screen.getByRole("form", { name: "Editar la regla de github.com" });
      const value = within(form).getByRole("textbox", { name: "google.com" });
      await userEvent.clear(value);
      await userEvent.type(value, "gitlab.com");
      await userEvent.click(within(form).getByRole("button", { name: "Guardar" }));

      expect(await within(form).findByRole("alert")).toHaveTextContent(/Ya hay una regla/);
      expect(value).toHaveValue("gitlab.com");
    });

    it("leaves the rule alone when the edit is cancelled", async () => {
      const form = await edit("github.com");
      await userEvent.click(within(form).getByRole("button", { name: "Cancelar" }));
      expect(screen.queryByRole("form")).toBeNull();
      expect(calls("rules_edit")).toHaveLength(0);
      expect(screen.getByTitle("github.com")).toBeInTheDocument();
    });

    it("greys the private box once the new browser has no private window", async () => {
      const form = await edit("https://mail.proton.me/inbox");
      const box = within(form).getByRole("checkbox", { name: "En ventana privada" });
      expect(box).toBeChecked();
      await userEvent.click(within(form).getByRole("button", { name: "Abrir en" }));
      await userEvent.click(within(form).getByRole("option", { name: /Plain Viewer/ }));
      expect(box).toBeDisabled();
      expect(box).not.toBeChecked();
    });
  });

  describe("testing a link", () => {
    const WINNER_PROBE = {
      winner: SITE,
      why: "narrower",
      also: [{ ...GONE, id: "host:github.com", covers: "github.com", kind: "host" as const }],
      site: "github.com",
    };

    async function open() {
      render(<Rules />);
      await userEvent.click(await screen.findByRole("button", { name: "Probar un enlace" }));
      return screen.getByRole("textbox", { name: "Enlace que quieres probar" });
    }

    afterEach(() => vi.useRealTimers());

    it("asks only once the typing has paused, and with the last thing typed", async () => {
      answers([SITE], { rules_probe: () => Promise.resolve(WINNER_PROBE) });
      const box = await open();
      vi.useFakeTimers();
      fireEvent.change(box, { target: { value: "git" } });
      await act(async () => {
        vi.advanceTimersByTime(150);
      });
      fireEvent.change(box, { target: { value: "github.com/x" } });
      await act(async () => {
        vi.advanceTimersByTime(150);
      });
      expect(calls("rules_probe")).toHaveLength(0);
      await act(async () => {
        vi.advanceTimersByTime(100);
      });
      expect(calls("rules_probe")).toEqual([
        ["rules_probe", { url: "github.com/x", fromApp: null }],
      ]);
    });

    it("says where the link would open and why that rule answers", async () => {
      answers([SITE, LINK], {
        rules_probe: () => Promise.resolve({ ...WINNER_PROBE, why: "alone", also: [] }),
      });
      const box = await open();
      await userEvent.type(box, "https://github.com/rgdevment");

      const status = await screen.findByRole("status");
      expect(status).toHaveTextContent("Se abriría en");
      expect(status).toHaveTextContent("Google Chrome");
      expect(status).toHaveTextContent("Tu Chrome");
      expect(status).toHaveTextContent("«Sitio · github.com»");
    });

    it("tells the reason apart for each way a rule can win", async () => {
      const reasons = {
        alone: "es la única regla que coincide",
        origin: "una regla «Desde» gana sobre las que miran la dirección",
        narrower: "es más específica que «Sitio · gitlab.com»",
        earlier: "es tan específica como «Sitio · gitlab.com» y está antes en la lista",
      };
      for (const [why, sentence] of Object.entries(reasons)) {
        invoke.mockReset();
        answers([SITE, GONE], {
          rules_probe: () =>
            Promise.resolve({ winner: SITE, why, also: why === "alone" ? [] : [GONE], site: "x" }),
        });
        const { unmount } = render(<Rules />);
        await userEvent.click(await screen.findByRole("button", { name: "Probar un enlace" }));
        await userEvent.type(
          screen.getByRole("textbox", { name: "Enlace que quieres probar" }),
          "a",
        );
        expect(await screen.findByText(new RegExp(sentence))).toBeInTheDocument();
        unmount();
      }
    });

    it("lifts the winner in the list and lists the others as also matching", async () => {
      answers([SITE, LINK, FROM_TEAMS, GONE], {
        rules_probe: () => Promise.resolve(WINNER_PROBE),
      });
      const box = await open();
      await userEvent.type(box, "github.com");
      await screen.findByRole("status");

      const row = within(theList()).getByTitle("github.com").closest("li") as HTMLElement;
      expect(within(row).getByText("Gana")).toBeInTheDocument();
      expect(row.className).toContain("bg-(--accent-soft)");
      expect(within(theList()).getAllByText("Gana")).toHaveLength(1);

      const matches = screen.getByRole("list", { name: "Reglas que coinciden" });
      expect(within(matches).getAllByRole("listitem")).toHaveLength(2);
      expect(within(matches).getByText("Gana")).toBeInTheDocument();
      expect(within(matches).getByText("También coincide")).toBeInTheDocument();
    });

    it("sends the application the link is said to come from", async () => {
      answers([SITE, FROM_TEAMS], { rules_probe: () => Promise.resolve(WINNER_PROBE) });
      const box = await open();
      await userEvent.type(box, "github.com");
      await userEvent.selectOptions(screen.getByRole("combobox", { name: "Desde" }), "teams");
      await vi.waitFor(() =>
        expect(invoke).toHaveBeenLastCalledWith("rules_probe", {
          url: "github.com",
          fromApp: "teams",
        }),
      );
    });

    it("offers no application to choose when no rule names one", async () => {
      answers([SITE], { rules_probe: () => Promise.resolve(WINNER_PROBE) });
      await open();
      expect(screen.queryByRole("combobox", { name: "Desde" })).toBeNull();
    });

    it("offers to create a rule for the site when nothing matches", async () => {
      answers([SITE], {
        rules_probe: () =>
          Promise.resolve({ winner: null, why: null, also: [], site: "example.org" }),
      });
      const box = await open();
      await userEvent.type(box, "https://docs.example.org/a");

      expect(await screen.findByText("Ninguna regla coincide")).toBeInTheDocument();
      expect(screen.getByText(/te preguntará dónde abrirlo/)).toBeInTheDocument();
      expect(screen.queryByRole("list", { name: "Reglas que coinciden" })).toBeNull();

      await userEvent.click(
        screen.getByRole("button", { name: "Crear una regla para example.org" }),
      );
      const form = screen.getByRole("form", { name: "Nueva regla" });
      expect(within(form).getByRole("textbox", { name: "google.com" })).toHaveValue("example.org");
      expect(within(form).getByRole("radio", { name: "Todo el sitio" })).toBeChecked();
    });

    it("warns that the winning rule can no longer open where it points", async () => {
      answers([GONE], {
        rules_probe: () =>
          Promise.resolve({ winner: GONE, why: "alone", also: [], site: "gitlab.com" }),
      });
      const box = await open();
      await userEvent.type(box, "gitlab.com");
      expect(await screen.findByText(/ya no puede abrir el enlace en brave/)).toBeInTheDocument();
      expect(screen.queryByText("Se abriría en")).toBeNull();
    });

    it("says why a link that is not one is refused, and clears the lifted rule", async () => {
      let refuse = false;
      answers([SITE], {
        rules_probe: () =>
          refuse ? Promise.reject("ruleValueNotUrl") : Promise.resolve(WINNER_PROBE),
      });
      const box = await open();
      await userEvent.type(box, "github.com");
      await screen.findByRole("status");
      expect(within(theList()).getByText("Gana")).toBeInTheDocument();

      refuse = true;
      await userEvent.type(box, " nope");
      expect(await screen.findByText(/Eso no es una dirección completa/)).toBeInTheDocument();
      expect(screen.queryByRole("status")).toBeNull();
      expect(within(theList()).queryByText("Gana")).toBeNull();
    });

    it("asks nothing for an empty box", async () => {
      answers([SITE], { rules_probe: () => Promise.resolve(WINNER_PROBE) });
      const box = await open();
      await userEvent.type(box, "a");
      await screen.findByRole("status");
      await userEvent.clear(box);
      await vi.waitFor(() => expect(screen.queryByRole("status")).toBeNull());
    });

    it("closes the panel, and the rule it lifted goes back to being one of the list", async () => {
      answers([SITE], { rules_probe: () => Promise.resolve(WINNER_PROBE) });
      const box = await open();
      await userEvent.type(box, "github.com");
      await screen.findByRole("status");

      await userEvent.click(screen.getByRole("button", { name: "Cerrar la prueba" }));
      expect(screen.queryByRole("textbox", { name: "Enlace que quieres probar" })).toBeNull();
      expect(screen.queryByText("Gana")).toBeNull();
    });

    it("closes with Escape and toggles from the same button that opened it", async () => {
      const box = await open();
      await userEvent.type(box, "{Escape}");
      expect(screen.queryByRole("textbox", { name: "Enlace que quieres probar" })).toBeNull();

      const opener = screen.getByRole("button", { name: "Probar un enlace" });
      await userEvent.click(opener);
      expect(opener).toHaveAttribute("aria-pressed", "true");
      await userEvent.click(opener);
      expect(opener).toHaveAttribute("aria-pressed", "false");
    });
  });
});
