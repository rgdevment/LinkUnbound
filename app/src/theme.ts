import { invoke } from "@tauri-apps/api/core";

type Theme = "system" | "light" | "dark";

const REMEMBERED = "theme-was-dark";

function paint(theme: Theme, systemIsDark: boolean) {
  const dark = theme === "dark" || (theme === "system" && systemIsDark);
  document.documentElement.classList.toggle("dark", dark);
  try {
    localStorage.setItem(REMEMBERED, dark ? "1" : "0");
  } catch {
    // Storage may be missing or blocked; the next open simply starts from the default again.
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

let reread: (() => void) | null = null;

export function follow() {
  const media = window.matchMedia("(prefers-color-scheme: dark)");
  let chosen: Theme = remembered();

  paint(chosen, media.matches);
  media.addEventListener("change", () => paint(chosen, media.matches));

  reread = () =>
    void invoke<{ prefs: { theme: Theme } }>("prefs_get")
      .then(({ prefs }) => {
        chosen = prefs.theme;
        paint(chosen, media.matches);
      })
      .catch(() => {});

  reread();
}

export function refresh() {
  reread?.();
}
