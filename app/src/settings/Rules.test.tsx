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
    hidden: false,
  },
  { id: "firefox", name: "Mozilla Firefox", profile_names: [], hidden: false },
  { id: "edge", name: "Microsoft Edge", profile_names: [["Default", "Personal"]], hidden: true },
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
