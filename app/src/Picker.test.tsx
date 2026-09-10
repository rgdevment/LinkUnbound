import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
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
        token: 1,
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
      token: 1,
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
      token: 1,
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
      token: 1,
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
          token: 1,
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
      token: 1,
    });
  });

  it("measures itself so the window is sized before it is shown", async () => {
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue({
      height: 210,
    } as DOMRect);
    render(<Picker />);
    await screen.findByText("Trabajo");
    expect(invoke).toHaveBeenCalledWith("picker_fit", { height: 210 });
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
      picker_open: () => Promise.reject("the profile Profile 2 is no longer there"),
    });
    render(<Picker />);
    await userEvent.click(await screen.findByText("Trabajo"));
    expect(await screen.findByText(/no longer there/)).toBeInTheDocument();
  });

  it("a shift plus digit chord opens that row in private mode even though shift turns the digit into a symbol", async () => {
    render(<Picker />);
    await screen.findByText("Trabajo");
    fireEvent.keyDown(window, { key: "!", code: "Digit1", shiftKey: true });
    expect(invoke).toHaveBeenCalledWith("picker_open", {
      browserId: "chrome",
      profileId: "Default",
      private: true,
      remember: "once",
      token: 1,
    });
  });

  it("the keyboard shortcut honours the private toggle, not just a held shift", async () => {
    render(<Picker />);
    await screen.findByText("Trabajo");
    await userEvent.click(screen.getByRole("button", { name: "Ventana privada" }));
    fireEvent.keyDown(window, { key: "2", code: "Digit2" });
    expect(invoke).toHaveBeenCalledWith("picker_open", {
      browserId: "chrome",
      profileId: "Profile 2",
      private: true,
      remember: "once",
      token: 1,
    });
  });

  it("the first destination receives focus once it loads, and arrow down moves to the next one", async () => {
    render(<Picker />);
    await screen.findByText("Trabajo");
    const items = screen.getAllByRole("menuitem");
    expect(document.activeElement).toBe(items[0]);
    fireEvent.keyDown(items[0], { key: "ArrowDown" });
    expect(document.activeElement).toBe(items[1]);
  });

  it("an empty destinations list still shows the picker instead of staying invisible forever", async () => {
    answers({ picker_destinations: () => Promise.resolve({ browsers: [], is_default: true }) });
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue({
      height: 90,
    } as DOMRect);
    render(<Picker />);
    await screen.findByText("gist.github.com");
    expect(invoke).toHaveBeenCalledWith("picker_fit", expect.anything());
  });

  it("a destinations request that fails surfaces a message instead of staying silent", async () => {
    answers({
      picker_destinations: () => Promise.reject("could not list the installed browsers"),
    });
    render(<Picker />);
    expect(await screen.findByText(/could not list the installed browsers/)).toBeInTheDocument();
  });

  it("a second incoming link resets remember and private mode, and asks the window to resize again", async () => {
    let deliver: ((event: { payload: unknown }) => void) | undefined;
    listen
      .mockReset()
      .mockImplementation((_event: string, cb: (event: { payload: unknown }) => void) => {
        deliver = cb;
        return Promise.resolve(() => {});
      });
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue({
      height: 200,
    } as DOMRect);

    render(<Picker />);
    await screen.findByText("Trabajo");
    await userEvent.click(screen.getByRole("radio", { name: "Todo el sitio" }));
    await userEvent.click(screen.getByRole("button", { name: "Ventana privada" }));
    invoke.mockClear();

    act(() => {
      deliver?.({
        payload: {
          url: "https://intranet.test/y",
          source_app: null,
          host: "intranet.test",
          site: "intranet.test",
          token: 1,
        },
      });
    });

    await waitFor(() => {
      expect(screen.getByRole("radio", { name: "Solo esta vez" })).toBeChecked();
    });
    expect(screen.getByRole("button", { name: "Ventana privada" })).toHaveAttribute(
      "aria-pressed",
      "false",
    );
    expect(invoke).toHaveBeenCalledWith("picker_fit", expect.anything());
  });

  it("the header shows the same host a saved rule would match, without the port the browser also carries", async () => {
    answers({
      picker_boot: () =>
        Promise.resolve({
          url: "https://intranet.test:8443/x",
          source_app: null,
          host: "intranet.test",
          site: "intranet.test",
          token: 1,
        }),
    });
    render(<Picker />);
    await screen.findByText(/intranet\.test/);
    expect(screen.getByText("intranet.test")).toBeInTheDocument();
  });
});
