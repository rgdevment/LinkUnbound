import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Line, Section } from "./parts";

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
});
