import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Picker from "./Picker";

const invoke = vi.fn();
const listen = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: (...a: unknown[]) => listen(...a) }));

const CHROME = {
  id: "chrome",
  name: "Chrome",
  exe: "chrome.exe",
  private_flag: "--incognito",
  icon: "data:image/png;base64,iVBORw0KGgo=",
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
  icon: null,
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
      return Promise.resolve({
        url: "https://gist.github.com/a",
        source_app: "slack",
        host: "gist.github.com",
        site: "github.com",
      });
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
    render(<Picker />);
    expect(await screen.findByText("Trabajo")).toBeInTheDocument();
    expect(screen.getByText("Personal")).toBeInTheDocument();
    expect(screen.getAllByText("Chrome")).toHaveLength(2);
    expect(screen.getByText("Firefox")).toBeInTheDocument();
  });

  it("shows the link the backend hands back and where it came from", async () => {
    render(<Picker />);
    expect(await screen.findByText("gist.github.com")).toBeInTheDocument();
    expect(screen.getByText("slack")).toBeInTheDocument();
  });

  it("asks the backend to open the chosen profile, and never sends the url", async () => {
    render(<Picker />);
    await userEvent.click(await screen.findByText("Trabajo"));
    expect(invoke).toHaveBeenCalledWith("picker_open", {
      browserId: "chrome",
      profileId: "Profile 2",
      private: false,
      remember: "once",
    });
  });

  it("a browser without profiles opens with no profile", async () => {
    render(<Picker />);
    await userEvent.click(await screen.findByText("Firefox"));
    expect(invoke).toHaveBeenCalledWith("picker_open", {
      browserId: "firefox",
      profileId: null,
      private: false,
      remember: "once",
    });
  });

  it("passes the chosen scope along so the rule gets saved", async () => {
    render(<Picker />);
    await userEvent.click(await screen.findByRole("radio", { name: "Todo el sitio" }));
    await userEvent.click(screen.getByText("Firefox"));
    expect(invoke).toHaveBeenCalledWith("picker_open", {
      browserId: "firefox",
      profileId: null,
      private: false,
      remember: "site",
    });
  });

  it("starts every link on the scope that writes nothing", async () => {
    render(<Picker />);
    const once = await screen.findByRole("radio", { name: "Solo esta vez" });
    expect(once).toBeChecked();
  });

  it("offers no subdomain scope when the host is already its own site", async () => {
    answers({
      picker_boot: () =>
        Promise.resolve({
          url: "https://github.com/a",
          source_app: null,
          host: "github.com",
          site: "github.com",
        }),
    });
    render(<Picker />);
    expect(await screen.findByRole("radio", { name: "Subdominio" })).toBeDisabled();
  });

  it("opens in a private window when shift is held", async () => {
    const user = userEvent.setup();
    render(<Picker />);
    const row = await screen.findByText("Trabajo");
    await user.keyboard("{Shift>}");
    await user.click(row);
    await user.keyboard("{/Shift}");
    expect(invoke).toHaveBeenCalledWith("picker_open", {
      browserId: "chrome",
      profileId: "Profile 2",
      private: true,
      remember: "once",
    });
  });

  it("measures itself so the window is sized before it is shown", async () => {
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue({
      height: 210,
    } as DOMRect);
    render(<Picker />);
    await screen.findByText("Trabajo");
    expect(invoke).toHaveBeenCalledWith("picker_fit", { height: 210 });
    vi.restoreAllMocks();
  });

  it("stays hidden rather than reporting a height it has not painted", async () => {
    render(<Picker />);
    await screen.findByText("Trabajo");
    expect(invoke).not.toHaveBeenCalledWith("picker_fit", expect.anything());
  });

  it("shows the real browser icon, and a placeholder when there is none", async () => {
    const { container } = render(<Picker />);
    await screen.findByText("Trabajo");
    const icons = container.querySelectorAll("img");
    expect(icons).toHaveLength(2);
    expect(icons[0].getAttribute("src")).toMatch(/^data:image\/png/);
    expect(icons[0].getAttribute("alt")).toBe("");
  });

  it("surfaces a refusal instead of pretending the link opened", async () => {
    answers({
      picker_open: () => Promise.reject(new Error("the profile Profile 2 is no longer there")),
    });
    render(<Picker />);
    await userEvent.click(await screen.findByText("Trabajo"));
    expect(await screen.findByText(/no longer there/)).toBeInTheDocument();
  });
});
