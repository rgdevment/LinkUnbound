import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";

const invoke = vi.fn();
const listen = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: (...a: unknown[]) => listen(...a) }));

const CHROME = {
  id: "chrome",
  name: "Chrome",
  exe: "chrome.exe",
  private_flag: "--incognito",
  profiles: [
    { id: "Default", name: "Personal" },
    { id: "Profile 2", name: "Trabajo" },
  ],
};

const FIREFOX = {
  id: "firefox",
  name: "Firefox",
  exe: "firefox.exe",
  private_flag: "-private-window",
  profiles: [],
};

/// Factories, not values: a rejected promise built up front is unhandled until
/// the component gets to it, and vitest reports that as a suite error.
function answers(overrides: Record<string, () => Promise<unknown>> = {}) {
  invoke.mockImplementation((cmd: string) => {
    if (cmd in overrides) return overrides[cmd]();
    if (cmd === "picker_destinations")
      return Promise.resolve({ browsers: [CHROME, FIREFOX], is_default: true });
    if (cmd === "picker_boot")
      return Promise.resolve({ url: "https://gist.github.com/a", source_app: "slack" });
    return Promise.resolve(null);
  });
}

describe("picker", () => {
  beforeEach(() => {
    invoke.mockReset();
    listen.mockReset().mockResolvedValue(() => {});
    answers();
  });

  it("offers one row per profile, not one per browser", async () => {
    render(<App />);
    expect(await screen.findByText("Trabajo")).toBeInTheDocument();
    expect(screen.getByText("Personal")).toBeInTheDocument();
    expect(screen.getAllByText("Chrome")).toHaveLength(2);
    expect(screen.getByText("Firefox")).toBeInTheDocument();
  });

  it("shows the link the backend hands back and where it came from", async () => {
    render(<App />);
    expect(await screen.findByText("gist.github.com")).toBeInTheDocument();
    expect(screen.getByText("slack")).toBeInTheDocument();
  });

  it("asks the backend to open the chosen profile, and never sends the url", async () => {
    render(<App />);
    await userEvent.click(await screen.findByText("Trabajo"));
    expect(invoke).toHaveBeenCalledWith("picker_open", {
      browserId: "chrome",
      profileId: "Profile 2",
      private: false,
      remember: false,
    });
  });

  it("a browser without profiles opens with no profile", async () => {
    render(<App />);
    await userEvent.click(await screen.findByText("Firefox"));
    expect(invoke).toHaveBeenCalledWith("picker_open", {
      browserId: "firefox",
      profileId: null,
      private: false,
      remember: false,
    });
  });

  it("passes the remember choice along so the rule gets saved", async () => {
    render(<App />);
    await userEvent.click(await screen.findByRole("checkbox"));
    await userEvent.click(screen.getByText("Firefox"));
    expect(invoke).toHaveBeenCalledWith("picker_open", {
      browserId: "firefox",
      profileId: null,
      private: false,
      remember: true,
    });
  });

  it("surfaces a refusal instead of pretending the link opened", async () => {
    answers({
      picker_open: () => Promise.reject(new Error("the profile Profile 2 is no longer there")),
    });
    render(<App />);
    await userEvent.click(await screen.findByText("Trabajo"));
    expect(await screen.findByText(/no longer there/)).toBeInTheDocument();
  });
});
