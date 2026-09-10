import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Notice from "./Notice";

const invoke = vi.fn();
const listen = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: (...a: unknown[]) => listen(...a) }));

const FIRED = {
  url: "https://github.com/rgdevment/LinkUnbound",
  rule_id: "site:github.com",
  browser: "Google Chrome",
};

let deliver: ((event: { payload: unknown }) => void) | undefined;

async function ticks(n: number) {
  for (let i = 0; i < n; i++) {
    await act(async () => {
      vi.advanceTimersByTime(1000);
    });
  }
}

async function fire(payload: unknown = FIRED) {
  await act(async () => {
    deliver?.({ payload });
  });
}

describe("notice", () => {
  beforeEach(() => {
    invoke.mockReset().mockResolvedValue(null);
    deliver = undefined;
    listen.mockReset().mockImplementation((_event: string, cb: typeof deliver) => {
      deliver = cb;
      return Promise.resolve(() => {});
    });
  });

  afterEach(() => vi.useRealTimers());

  it("shows nothing until a rule actually fires", () => {
    const { container } = render(<Notice />);
    expect(container).toBeEmptyDOMElement();
  });

  /// The window is built after the event would have been emitted, so the notice
  /// pulls what fired instead of waiting to be told.
  it("picks up a rule that fired before it existed", async () => {
    invoke.mockImplementation((cmd: string) =>
      Promise.resolve(cmd === "notice_boot" ? FIRED : null),
    );
    render(<Notice />);
    expect(await screen.findByText("Google Chrome")).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith("notice_ready");
  });

  it("names the browser a rule chose and the host it chose it for", async () => {
    render(<Notice />);
    await fire();
    expect(screen.getByText("Google Chrome")).toBeInTheDocument();
    expect(screen.getByText(/github\.com/)).toBeInTheDocument();
  });

  /// Undo has to remove the rule: keeping it would ask the same question on the
  /// next link, and pressing undo already answered it.
  it("undoing names the rule that fired, so the right one is removed", async () => {
    render(<Notice />);
    await fire();
    await userEvent.click(screen.getByRole("button", { name: "Deshacer" }));
    expect(invoke).toHaveBeenCalledWith("notice_undo", { ruleId: "site:github.com" });
  });

  it("can be dismissed by hand before the countdown ends", async () => {
    render(<Notice />);
    await fire();
    await userEvent.click(screen.getByRole("button", { name: "Cerrar el aviso" }));
    expect(invoke).toHaveBeenCalledWith("notice_dismiss");
  });

  it("closes itself once the countdown runs out", async () => {
    vi.useFakeTimers();
    render(<Notice />);
    await fire();
    await ticks(7);
    expect(invoke).toHaveBeenCalledWith("notice_dismiss");
  });

  /// A second link arriving while the notice is up must restart the countdown,
  /// or it would vanish before the user reads the newer one.
  it("restarts the countdown when another rule fires", async () => {
    vi.useFakeTimers();
    render(<Notice />);
    await fire();
    await ticks(4);
    await fire({ ...FIRED, browser: "Mozilla Firefox", rule_id: "site:other.test" });
    await ticks(4);
    expect(invoke).not.toHaveBeenCalledWith("notice_dismiss");
    expect(screen.getByText("Mozilla Firefox")).toBeInTheDocument();
  });
});
