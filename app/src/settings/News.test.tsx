import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Speaking } from "../i18n";
import News from "./News";

const opened: string[] = [];
const invoke = vi.fn();

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: (url: string) => {
    opened.push(url);
    return Promise.resolve();
  },
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

function waiting(versions: unknown) {
  invoke.mockImplementation((cmd: string) =>
    Promise.resolve(cmd === "news" ? versions : undefined),
  );
}

beforeEach(() => {
  invoke.mockReset();
  opened.length = 0;
});

describe("the news after an update", () => {
  it("says nothing when nothing is waiting", async () => {
    waiting([]);
    render(<News />);
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("news"));
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("ignores an answer that is not a list", async () => {
    waiting({ registered: true });
    render(<News />);
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("news"));
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("tells the newest version first under its own headline, with the bear above", async () => {
    waiting(["2.6.0", "2.5.0"]);
    render(<News />);
    const dialog = await screen.findByRole("dialog", { name: "LinkUnbound estrena mascota" });
    expect(dialog.querySelector("img")).toBeTruthy();
    expect(screen.getByText("Un oso que cuida tus enlaces")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "2.5.0" })).toBeInTheDocument();
  });

  it("speaks the language the window speaks", async () => {
    waiting(["2.6.0"]);
    render(
      <Speaking value="en">
        <News />
      </Speaking>,
    );
    expect(await screen.findByRole("dialog", { name: "LinkUnbound has a mascot" })).toBeTruthy();
  });

  it("is written down as told once closed, and goes away", async () => {
    waiting(["2.6.0"]);
    render(<News />);
    await userEvent.click(await screen.findByRole("button", { name: "Entendido" }));
    expect(invoke).toHaveBeenCalledWith("news_seen");
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("closes with Escape too", async () => {
    waiting(["2.6.0"]);
    render(<News />);
    await screen.findByRole("dialog");
    await userEvent.keyboard("{Escape}");
    expect(invoke).toHaveBeenCalledWith("news_seen");
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("opens the releases page for anyone who wants every version", async () => {
    waiting(["2.6.0"]);
    render(<News />);
    await userEvent.click(await screen.findByRole("button", { name: "Ver todas las versiones" }));
    expect(opened).toEqual(["https://github.com/rgdevment/LinkUnbound/releases"]);
  });

  it("shows nothing for a version the news does not know", async () => {
    waiting(["9.9.9"]);
    render(<News />);
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("news"));
    expect(screen.queryByRole("dialog")).toBeNull();
  });
});

describe("the news keeps the keyboard", () => {
  beforeEach(() => {
    invoke.mockReset();
    waiting(["2.6.0"]);
  });

  it("closes with Escape wherever the focus wandered", async () => {
    render(
      <>
        <button type="button">behind</button>
        <News />
      </>,
    );
    await screen.findByRole("dialog");
    screen.getByRole("button", { name: "behind" }).focus();
    await userEvent.keyboard("{Escape}");
    expect(invoke).toHaveBeenCalledWith("news_seen");
  });

  it("cycles Tab between its own buttons", async () => {
    render(
      <>
        <button type="button">behind</button>
        <News />
      </>,
    );
    const ok = await screen.findByRole("button", { name: "Entendido" });
    expect(ok).toHaveFocus();
    await userEvent.tab();
    expect(screen.getByRole("button", { name: "Ver todas las versiones" })).toHaveFocus();
    await userEvent.tab({ shift: true });
    expect(ok).toHaveFocus();
  });
});
