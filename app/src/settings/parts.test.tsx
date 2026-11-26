import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Choice, Line, Section, Switch } from "./parts";

describe("settings text", () => {
  it("draws a section title in the muted colour at the smallest size", () => {
    render(<Section title="Startup">body</Section>);
    const title = screen.getByRole("heading", { name: "Startup" });
    expect(title).toHaveClass("text-muted", "text-xs");
  });

  it("draws a line note in the muted colour one step above the smallest size", () => {
    render(<Line title="Theme" note="Follows the system" />);
    const note = screen.getByText("Follows the system");
    expect(note).toHaveClass("text-muted", "text-sm");
    expect(note).not.toHaveClass("text-xs");
    expect(screen.getByText("Theme")).toHaveClass("text-md");
  });

  it("gives an enabled choice chip hover classes and a disabled one none", () => {
    const options = [
      { id: "a", label: "lookSize" as const },
      { id: "b", label: "lookContrast" as const },
    ];
    const pick = () => {};
    const { rerender } = render(<Choice options={options} value="a" label="Pick" onPick={pick} />);
    const chosen = screen.getByText("Tamaño del texto");
    const other = screen.getByText("Contraste alto");
    expect(chosen).toHaveClass("hover:bg-accent/[0.2]");
    expect(other).toHaveClass("hover:bg-black/[0.09]", "hover:text-ink");
    rerender(<Choice options={options} value="a" label="Pick" disabled onPick={pick} />);
    for (const chip of [chosen, other]) {
      expect(chip.className).not.toContain("hover:");
    }
  });

  it("gives a switch enabled-only hover classes", () => {
    render(<Switch on={false} label="Startup" onChange={() => {}} />);
    const toggle = screen.getByRole("switch", { name: "Startup" });
    expect(toggle).toHaveClass("enabled:hover:ring-4", "enabled:hover:bg-ink-2/40");
  });

  it("positions the choice fieldset so its hidden radios stay inside the scroller", () => {
    const options = [{ id: "a", label: "lookSize" as const }];
    render(<Choice options={options} value="a" label="Pick" onPick={() => {}} />);
    expect(screen.getByRole("group", { name: "Pick" })).toHaveClass("relative");
  });
});
