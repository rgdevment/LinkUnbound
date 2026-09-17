import { beforeEach, describe, expect, it, vi } from "vitest";
import { follow, refresh } from "./theme";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

type Listener = () => void;

function systemPrefers(dark: boolean) {
  const listeners: Listener[] = [];
  const media = {
    matches: dark,
    addEventListener: (event: string, listener: Listener) => {
      if (event === "change") listeners.push(listener);
    },
  };
  const asked = vi.fn().mockReturnValue(media);
  window.matchMedia = asked as unknown as typeof window.matchMedia;
  return {
    asked,
    flip(nowDark: boolean) {
      media.matches = nowDark;
      for (const listener of listeners) listener();
    },
  };
}

function isDark() {
  return document.documentElement.classList.contains("dark");
}

async function settled() {
  await new Promise((resolve) => setTimeout(resolve, 0));
}

describe("theme", () => {
  beforeEach(() => {
    document.documentElement.classList.remove("dark");
    localStorage.clear();
    invoke.mockReset();
  });

  it("is dark before the backend has said anything", () => {
    systemPrefers(false);
    invoke.mockReturnValue(new Promise(() => {}));
    follow();
    expect(isDark()).toBe(true);
  });

  /// A person on the light theme saw a dark frame at every open, before the backend answered.
  it("starts from what it painted last time", async () => {
    systemPrefers(false);
    invoke.mockResolvedValue({ prefs: { theme: "light" } });
    follow();
    await settled();
    expect(isDark()).toBe(false);

    document.documentElement.classList.remove("dark");
    invoke.mockReturnValue(new Promise(() => {}));
    follow();
    expect(isDark()).toBe(false);
  });

  it("follows what the backend chose", async () => {
    systemPrefers(true);
    invoke.mockResolvedValue({ prefs: { theme: "light" } });
    follow();
    await settled();
    expect(isDark()).toBe(false);
    expect(invoke).toHaveBeenCalledWith("prefs_get");
  });

  it("follows the system only when told to, and keeps following it", async () => {
    const system = systemPrefers(false);
    invoke.mockResolvedValue({ prefs: { theme: "system" } });
    follow();
    await settled();
    expect(system.asked).toHaveBeenCalledWith("(prefers-color-scheme: dark)");
    expect(isDark()).toBe(false);
    system.flip(true);
    expect(isDark()).toBe(true);
    system.flip(false);
    expect(isDark()).toBe(false);
  });

  it("ignores the system once a side was chosen", async () => {
    const system = systemPrefers(false);
    invoke.mockResolvedValue({ prefs: { theme: "dark" } });
    follow();
    await settled();
    system.flip(false);
    expect(isDark()).toBe(true);
  });

  it("asks again on refresh and repaints", async () => {
    systemPrefers(false);
    invoke.mockResolvedValueOnce({ prefs: { theme: "light" } });
    follow();
    await settled();
    expect(isDark()).toBe(false);
    invoke.mockResolvedValueOnce({ prefs: { theme: "dark" } });
    refresh();
    await settled();
    expect(isDark()).toBe(true);
    expect(invoke).toHaveBeenCalledTimes(2);
  });

  it("has nothing to refresh before anyone follows", async () => {
    vi.resetModules();
    const fresh = await import("./theme");
    expect(() => fresh.refresh()).not.toThrow();
    expect(invoke).not.toHaveBeenCalled();
  });

  it("survives a backend that answers with an error", async () => {
    systemPrefers(false);
    invoke.mockRejectedValue(new Error("no bridge"));
    follow();
    await settled();
    expect(isDark()).toBe(true);
  });
});
