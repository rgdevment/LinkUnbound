import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import RuleForm, { draftOf } from "./RuleForm";
import type { Destination, RuleView } from "./types";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const CHOICES: Destination[] = [
  { browserId: "chrome", profileId: null, label: "Chrome", icon: null, opensPrivately: true },
  { browserId: "plain", profileId: null, label: "Plain", icon: null, opensPrivately: false },
];

const RULE: RuleView = {
  id: "site:github.com",
  browser_id: "gone",
  profile_id: null,
  kind: "site",
  covers: "github.com",
  browser: "Brave",
  profile: null,
  icon: null,
  private: false,
  source_app: null,
  resolved: false,
  profile_gone: false,
};

describe("rule form", () => {
  beforeEach(() => invoke.mockReset());

  it("starts from what a rule says, as the person reads it", () => {
    expect(draftOf(RULE)).toEqual({ kind: "site", value: "github.com" });
    expect(draftOf({ ...RULE, kind: "host", covers: "a.test" })).toEqual({
      kind: "host",
      value: "a.test",
    });
    expect(draftOf({ ...RULE, kind: "any", covers: "", source_app: "teams" })).toEqual({
      kind: "app",
      value: "teams",
    });
    expect(draftOf({ ...RULE, kind: "any", covers: "" })).toEqual({ kind: "site", value: "" });
  });

  it("gives save, cancel and the kind segments a hover, and the buttons a focus ring", () => {
    render(<RuleForm choices={CHOICES} onSaved={vi.fn()} onCancel={vi.fn()} />);
    for (const name of ["Guardar", "Cancelar"]) {
      const button = screen.getByRole("button", { name });
      expect(button.className).toContain("hover:");
      expect(button.className).toContain("focus-visible:outline");
    }
    expect(screen.getByRole("radio", { name: "Este dominio" }).closest("label")).toHaveClass(
      "hover:text-ink",
    );
  });

  it("takes the first browser it is offered and puts the cursor in the value", () => {
    render(<RuleForm choices={CHOICES} onSaved={vi.fn()} onCancel={vi.fn()} />);
    expect(screen.getByRole("button", { name: "Abrir en" })).toHaveTextContent("Chrome");
    expect(screen.getByRole("textbox", { name: "google.com" })).toHaveFocus();
  });

  it("takes the first browser when the catalogue lands after the form opened", () => {
    const { rerender } = render(<RuleForm choices={[]} onSaved={vi.fn()} onCancel={vi.fn()} />);
    expect(screen.getByText("Windows no reporta ningún navegador instalado.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Guardar" })).toBeDisabled();
    rerender(<RuleForm choices={CHOICES} onSaved={vi.fn()} onCancel={vi.fn()} />);
    expect(screen.getByRole("button", { name: "Abrir en" })).toHaveTextContent("Chrome");
    expect(screen.getByRole("button", { name: "Guardar" })).toBeEnabled();
  });

  it("starts the kind from the draft it was given", () => {
    render(
      <RuleForm
        choices={CHOICES}
        start={{ kind: "url", value: "https://a.test/x" }}
        onSaved={vi.fn()}
        onCancel={vi.fn()}
      />,
    );
    expect(screen.getByRole("radio", { name: "Esta URL exacta" })).toBeChecked();
    expect(screen.getByRole("textbox", { name: /https:\/\/docs/ })).toHaveValue("https://a.test/x");
  });

  it("chooses the kind with the arrow keys, as a radio group does", async () => {
    render(<RuleForm choices={CHOICES} onSaved={vi.fn()} onCancel={vi.fn()} />);
    screen.getByRole("radio", { name: "Todo el sitio" }).focus();
    await userEvent.keyboard("{ArrowRight}");
    expect(screen.getByRole("radio", { name: "Este dominio" })).toBeChecked();
  });

  it("names a destination the catalogue no longer offers, and warns", () => {
    render(
      <RuleForm
        choices={CHOICES}
        editing={RULE}
        start={draftOf(RULE)}
        onSaved={vi.fn()}
        onCancel={vi.fn()}
      />,
    );
    expect(screen.getByRole("button", { name: "Abrir en" })).toHaveTextContent("Brave");
  });

  it("does not send a private window the chosen browser cannot open", async () => {
    invoke.mockResolvedValue([]);
    render(
      <RuleForm
        choices={CHOICES}
        start={{ kind: "site", value: "a.test" }}
        editing={{ ...RULE, browser_id: "chrome", private: true, resolved: true }}
        onSaved={vi.fn()}
        onCancel={vi.fn()}
      />,
    );
    await userEvent.click(screen.getByRole("button", { name: "Abrir en" }));
    await userEvent.click(screen.getByRole("option", { name: /Plain/ }));
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    expect(invoke).toHaveBeenCalledWith("rules_edit", {
      id: "site:github.com",
      kind: "site",
      value: "a.test",
      browserId: "plain",
      profileId: null,
      private: false,
    });
  });

  it("hands the list that came back to whoever opened it", async () => {
    invoke.mockResolvedValue([RULE]);
    const onSaved = vi.fn();
    render(<RuleForm choices={CHOICES} onSaved={onSaved} onCancel={vi.fn()} />);
    await userEvent.type(screen.getByRole("textbox", { name: "google.com" }), "a.test");
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    await vi.waitFor(() => expect(onSaved).toHaveBeenCalledWith([RULE]));
  });

  it("words a refusal it has no sentence for, and can be tried again", async () => {
    invoke.mockRejectedValueOnce("something odd").mockResolvedValueOnce([RULE]);
    const onSaved = vi.fn();
    render(<RuleForm choices={CHOICES} onSaved={onSaved} onCancel={vi.fn()} />);
    await userEvent.type(screen.getByRole("textbox", { name: "google.com" }), "a.test");
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("something odd");
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    await vi.waitFor(() => expect(onSaved).toHaveBeenCalled());
    expect(screen.queryByRole("alert")).toBeNull();
  });
});
