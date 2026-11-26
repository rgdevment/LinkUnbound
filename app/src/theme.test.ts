import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
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

  it("follows the system before the backend has said anything", () => {
    systemPrefers(false);
    invoke.mockReturnValue(new Promise(() => {}));
    follow();
    expect(isDark()).toBe(false);
  });

  it("follows a dark system too, with nothing remembered", () => {
    systemPrefers(true);
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
    expect(isDark()).toBe(false);
  });

  describe("contrast and text size", () => {
    const classes = () =>
      ["contrast", "text-small", "text-large"].filter((c) =>
        document.documentElement.classList.contains(c),
      );

    beforeEach(() => {
      document.documentElement.classList.remove("contrast", "text-small", "text-large");
      systemPrefers(false);
    });

    it("dresses the window in what the backend chose", async () => {
      invoke.mockResolvedValue({
        prefs: { theme: "light", high_contrast: true, text_size: "large" },
      });
      follow();
      await settled();
      expect(classes()).toEqual(["contrast", "text-large"]);
    });

    it("marks the small size on its own", async () => {
      invoke.mockResolvedValue({ prefs: { theme: "light", text_size: "small" } });
      follow();
      await settled();
      expect(classes()).toEqual(["text-small"]);
    });

    it("adds no class when the backend sends neither field", async () => {
      invoke.mockResolvedValue({ prefs: { theme: "light" } });
      follow();
      await settled();
      expect(classes()).toEqual([]);
    });

    it("takes the classes off again when they are turned off", async () => {
      invoke.mockResolvedValueOnce({
        prefs: { theme: "light", high_contrast: true, text_size: "small" },
      });
      follow();
      await settled();
      invoke.mockResolvedValueOnce({
        prefs: { theme: "light", high_contrast: false, text_size: "normal" },
      });
      refresh();
      await settled();
      expect(classes()).toEqual([]);
    });

    it("only a true contrast counts", async () => {
      invoke.mockResolvedValue({ prefs: { theme: "light", high_contrast: "yes" } });
      follow();
      await settled();
      expect(classes()).toEqual([]);
    });

    it("remembers it for the first frame of the next open", async () => {
      invoke.mockResolvedValue({
        prefs: { theme: "light", high_contrast: true, text_size: "large" },
      });
      follow();
      await settled();
      expect(JSON.parse(localStorage.getItem("look-was") ?? "null")).toEqual({
        contrast: true,
        size: "large",
      });

      document.documentElement.classList.remove("contrast", "text-large");
      invoke.mockReturnValue(new Promise(() => {}));
      follow();
      expect(classes()).toEqual(["contrast", "text-large"]);
    });

    it("starts from the remembered look before the backend answers", () => {
      localStorage.setItem("look-was", JSON.stringify({ contrast: false, size: "small" }));
      invoke.mockReturnValue(new Promise(() => {}));
      follow();
      expect(classes()).toEqual(["text-small"]);
    });

    describe("on the welcome", () => {
      beforeEach(() => {
        window.location.hash = "#welcome";
      });
      afterEach(() => {
        window.location.hash = "";
      });

      it("keeps the contrast but never scales the text", async () => {
        invoke.mockResolvedValue({
          prefs: { theme: "light", high_contrast: true, text_size: "large" },
        });
        follow();
        await settled();
        expect(classes()).toEqual(["contrast"]);
      });

      it("does not scale it from the remembered look either", () => {
        localStorage.setItem("look-was", JSON.stringify({ contrast: true, size: "small" }));
        invoke.mockReturnValue(new Promise(() => {}));
        follow();
        expect(classes()).toEqual(["contrast"]);
      });

      it("still remembers the size for the other windows", async () => {
        invoke.mockResolvedValue({ prefs: { theme: "light", text_size: "large" } });
        follow();
        await settled();
        expect(JSON.parse(localStorage.getItem("look-was") ?? "null")).toEqual({
          contrast: false,
          size: "large",
        });
      });
    });

    it.each([
      ["malformed storage", "{not json"],
      ["a null", "null"],
      ["missing fields", "{}"],
      ["wrong types", JSON.stringify({ contrast: "yes", size: 3 })],
      ["an unknown size", JSON.stringify({ contrast: false, size: "huge" })],
    ])("adds no class from %s", (_what, stored) => {
      localStorage.setItem("look-was", stored);
      invoke.mockReturnValue(new Promise(() => {}));
      follow();
      expect(classes()).toEqual([]);
    });
  });
});
