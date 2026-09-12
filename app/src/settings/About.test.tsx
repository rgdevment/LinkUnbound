import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import About from "./About";

describe("about", () => {
  it("names the licence, the coffee and the sibling tool", async () => {
    render(<About />);
    expect(screen.getByText("Licencia GPL-3.0")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Invítame un café" })).toBeInTheDocument();
    expect(screen.getByText("CopyPaste")).toBeInTheDocument();
  });

  it("points each one at where it really lives", async () => {
    render(<About />);
    const links = screen.getAllByRole("link").map((a) => a.getAttribute("href"));
    expect(links).toContain("https://buymeacoffee.com/rgdevment");
    expect(links).toContain("https://github.com/rgdevment/CopyPaste");
    expect(links).toContain("https://github.com/rgdevment/LinkUnbound/blob/main/LICENSE");
  });

  /// A link opening inside the webview would trap the user with no way back.
  it("opens every link outside the settings window", async () => {
    render(<About />);
    for (const link of screen.getAllByRole("link")) {
      expect(link).toHaveAttribute("target", "_blank");
      expect(link).toHaveAttribute("rel", expect.stringContaining("noreferrer"));
    }
  });
});
