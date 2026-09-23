import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Star from "./Star";

const opened: string[] = [];
const invoke = vi.fn();

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: (url: string) => {
    opened.push(url);
    return Promise.resolve();
  },
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const settled = vi.fn();

function said(name: string) {
  return screen.getByRole("button", { name });
}

describe("the card that asks for a star", () => {
  beforeEach(() => {
    opened.length = 0;
    invoke.mockReset();
    invoke.mockResolvedValue(null);
    settled.mockReset();
    render(<Star onSettled={settled} />);
  });

  it("opens the repository and is never asked for again", async () => {
    await userEvent.click(said("Dar una estrella"));

    expect(opened).toEqual(["https://github.com/rgdevment/LinkUnbound"]);
    expect(invoke).toHaveBeenCalledWith("star_done");
    expect(settled).toHaveBeenCalled();
  });

  it("is retired without opening anything when the answer is no", async () => {
    await userEvent.click(said("No mostrar más"));

    expect(opened).toEqual([]);
    expect(invoke).toHaveBeenCalledWith("star_done");
    expect(settled).toHaveBeenCalled();
  });

  it("leaves the question open when it is dismissed for now", async () => {
    await userEvent.click(said("Ahora no"));

    expect(opened).toEqual([]);
    expect(invoke).not.toHaveBeenCalled();
    expect(settled).toHaveBeenCalled();
  });
});
