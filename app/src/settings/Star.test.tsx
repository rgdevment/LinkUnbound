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
    render(<Star store={false} onSettled={settled} />);
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

/// The star does nothing for a copy the Store sold: what decides whether anybody searching the
/// Store ever sees this is the rating, and that is what such a copy has to be asked for.
describe("the same card in a copy the Store sold", () => {
  beforeEach(() => {
    opened.length = 0;
    invoke.mockReset();
    invoke.mockResolvedValue(null);
    settled.mockReset();
    render(<Star store onSettled={settled} />);
  });

  it("asks for the rating, and opens the Store rather than the repository", async () => {
    expect(screen.queryByRole("button", { name: "Dar una estrella" })).toBeNull();
    await userEvent.click(said("Valorar en la Store"));

    expect(opened).toEqual(["ms-windows-store://review/?ProductId=9N9F7C8Q43KC"]);
    expect(invoke).toHaveBeenCalledWith("star_done");
    expect(settled).toHaveBeenCalled();
  });

  it("is retired the same way when the answer is no", async () => {
    await userEvent.click(said("No mostrar más"));

    expect(opened).toEqual([]);
    expect(invoke).toHaveBeenCalledWith("star_done");
  });
});
