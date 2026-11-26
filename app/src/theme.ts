import { invoke } from "@tauri-apps/api/core";

type Theme = "system" | "light" | "dark";
type TextSize = "small" | "normal" | "large";

type Look = { theme: Theme; high_contrast?: boolean; text_size?: TextSize; font?: string | null };

type Dressed = { contrast: boolean; size: TextSize; font: string | null };

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

/// A family name as CSS reads it, whatever quotes or backslashes it carries.
export function familyCss(family: string): string {
  return `"${family.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

function dress({ contrast, size, font }: Dressed) {
  const root = document.documentElement;
  // The welcome is a fixed 600×460 with no scrolling, laid out for the base scale and font.
  const scales = window.location.hash !== "#welcome";
  root.classList.toggle("contrast", contrast);
  root.classList.toggle("text-small", scales && size === "small");
  root.classList.toggle("text-large", scales && size === "large");
  if (scales && font) {
    root.style.setProperty("--worn-font", `${familyCss(font)}, "Segoe UI Variable Text"`);
  } else {
    root.style.removeProperty("--worn-font");
  }
  try {
    localStorage.setItem(REMEMBERED_LOOK, JSON.stringify({ contrast, size, font }));
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

function rememberedLook(): Dressed {
  try {
    const was = JSON.parse(localStorage.getItem(REMEMBERED_LOOK) ?? "null") as {
      contrast?: unknown;
      size?: unknown;
      font?: unknown;
    } | null;
    const size = was?.size === "small" || was?.size === "large" ? was.size : "normal";
    const font = typeof was?.font === "string" && was.font ? was.font : null;
    return { contrast: was?.contrast === true, size, font };
  } catch {
    return { contrast: false, size: "normal", font: null };
  }
}

/// `system` and blank mean the system's own, as the resident reads them.
function chosenFont(font: string | null | undefined): string | null {
  const family = font?.trim();
  return family && family.toLowerCase() !== "system" ? family : null;
}

let reread: (() => void) | null = null;

export function follow() {
  const media = window.matchMedia("(prefers-color-scheme: dark)");
  let chosen: Theme = remembered();
  const was = rememberedLook();

  paint(chosen, media.matches);
  dress(was);
  media.addEventListener("change", () => paint(chosen, media.matches));

  reread = () =>
    void invoke<{ prefs: Look }>("prefs_get")
      .then(({ prefs }) => {
        chosen = prefs.theme;
        paint(chosen, media.matches);
        dress({
          contrast: prefs.high_contrast === true,
          size: prefs.text_size ?? "normal",
          font: chosenFont(prefs.font),
        });
      })
      .catch(() => {});

  reread();
}

export function refresh() {
  reread?.();
}
