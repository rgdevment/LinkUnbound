import { invoke } from "@tauri-apps/api/core";

type Theme = "system" | "light" | "dark";
type TextSize = "small" | "normal" | "large";

type Look = { theme: Theme; high_contrast?: boolean; text_size?: TextSize };

const REMEMBERED = "theme-was-dark";
const REMEMBERED_LOOK = "look-was";

function paint(theme: Theme, systemIsDark: boolean) {
  const dark = theme === "dark" || (theme === "system" && systemIsDark);
  document.documentElement.classList.toggle("dark", dark);
  try {
    localStorage.setItem(REMEMBERED, dark ? "1" : "0");
  } catch {
    // Storage may be missing or blocked; the next open simply starts from the default again.
  }
}

function dress(contrast: boolean, size: TextSize) {
  const root = document.documentElement.classList;
  // The welcome is a fixed 600×460 with no scrolling, laid out for the base scale.
  const scales = window.location.hash !== "#welcome";
  root.toggle("contrast", contrast);
  root.toggle("text-small", scales && size === "small");
  root.toggle("text-large", scales && size === "large");
  try {
    localStorage.setItem(REMEMBERED_LOOK, JSON.stringify({ contrast, size }));
  } catch {
    // As above.
  }
}

/// What was painted last time, so the first frame is not the default flashing before the
/// choice arrives. With nothing remembered it follows the system, as a fresh install does.
function remembered(): Theme {
  try {
    const was = localStorage.getItem(REMEMBERED);
    if (was === "0") return "light";
    if (was === "1") return "dark";
  } catch {
    // As above.
  }
  return "system";
}

function rememberedLook(): { contrast: boolean; size: TextSize } {
  try {
    const was = JSON.parse(localStorage.getItem(REMEMBERED_LOOK) ?? "null") as {
      contrast?: unknown;
      size?: unknown;
    } | null;
    const size = was?.size === "small" || was?.size === "large" ? was.size : "normal";
    return { contrast: was?.contrast === true, size };
  } catch {
    return { contrast: false, size: "normal" };
  }
}

let reread: (() => void) | null = null;

export function follow() {
  const media = window.matchMedia("(prefers-color-scheme: dark)");
  let chosen: Theme = remembered();
  const was = rememberedLook();

  paint(chosen, media.matches);
  dress(was.contrast, was.size);
  media.addEventListener("change", () => paint(chosen, media.matches));

  reread = () =>
    void invoke<{ prefs: Look }>("prefs_get")
      .then(({ prefs }) => {
        chosen = prefs.theme;
        paint(chosen, media.matches);
        dress(prefs.high_contrast === true, prefs.text_size ?? "normal");
      })
      .catch(() => {});

  reread();
}

export function refresh() {
  reread?.();
}
