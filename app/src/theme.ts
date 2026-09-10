import { invoke } from "@tauri-apps/api/core";

type Theme = "system" | "light" | "dark";

function paint(theme: Theme, systemIsDark: boolean) {
  const dark = theme === "dark" || (theme === "system" && systemIsDark);
  document.documentElement.classList.toggle("dark", dark);
}

/// Both windows call this: the theme is one choice and it has to look the same
/// in the picker as in settings.
export function follow() {
  const media = window.matchMedia("(prefers-color-scheme: dark)");
  let chosen: Theme = "system";

  paint(chosen, media.matches);
  media.addEventListener("change", () => paint(chosen, media.matches));

  const read = () =>
    void invoke<{ prefs: { theme: Theme } }>("prefs_get")
      .then(({ prefs }) => {
        chosen = prefs.theme;
        paint(chosen, media.matches);
      })
      .catch(() => {});

  read();
  return read;
}
