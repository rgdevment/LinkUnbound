import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Rules from "./Rules";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const SITE = {
  id: "site:github.com",
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
  kind: "any" as const,
  covers: "",
  browser: "Microsoft Edge",
  profile: "Trabajo",
  icon: null,
  private: false,
  source_app: "teams",
  resolved: true,
};

function answers(list: unknown[], overrides: Record<string, () => Promise<unknown>> = {}) {
  invoke.mockImplementation((cmd: string) => {
    if (cmd in overrides) return overrides[cmd]();
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
    expect(screen.getByText("Google Chrome")).toBeInTheDocument();
    expect(screen.getByText("Tu Chrome")).toBeInTheDocument();
  });

  /// A rule carried over from 1.4 answers only links from one app, which is not
  /// something the picker can create any more.
  it("shows a rule bound to an origin by the app it came from", async () => {
    render(<Rules />);
    expect(await screen.findByText("teams")).toBeInTheDocument();
    expect(screen.getByText("Desde")).toBeInTheDocument();
  });

  it("marks the rules that open in a private window", async () => {
    render(<Rules />);
    expect(await screen.findByText("privada")).toBeInTheDocument();
  });

  it("removes only the rule whose button was pressed", async () => {
    render(<Rules />);
    await userEvent.click(
      await screen.findByRole("button", { name: "Eliminar la regla de github.com" }),
    );
    expect(invoke).toHaveBeenCalledWith("rules_remove", { id: "site:github.com" });
  });

  it("sends the whole order when a rule moves, so ties resolve the way they read", async () => {
    render(<Rules />);
    await userEvent.click(
      await screen.findByRole("button", { name: "Bajar la regla de github.com" }),
    );
    expect(invoke).toHaveBeenCalledWith("rules_reorder", {
      ids: [LINK.id, SITE.id, FROM_TEAMS.id],
    });
  });

  it("cannot move the first rule up nor the last one down", async () => {
    render(<Rules />);
    expect(
      await screen.findByRole("button", { name: "Subir la regla de github.com" }),
    ).toBeDisabled();
    expect(screen.getByRole("button", { name: "Bajar la regla de teams" })).toBeDisabled();
  });

  it("surfaces a broken store instead of showing no rules at all", async () => {
    answers([], { rules_list: () => Promise.reject("rules.json could not be read") });
    render(<Rules />);
    expect(await screen.findByText(/could not be read/)).toBeInTheDocument();
  });

  /// A browser that was uninstalled leaves its rules pointing nowhere; striking
  /// the name is what tells the user why nothing happens.
  it("strikes through a destination that is no longer installed", async () => {
    answers([{ ...SITE, resolved: false, browser: "brave" }]);
    render(<Rules />);
    expect(await screen.findByText("brave")).toHaveClass("line-through");
  });
});
