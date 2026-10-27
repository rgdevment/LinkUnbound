import { invoke } from "@tauri-apps/api/core";
import { useCallback, useState } from "react";
import { useSpoken } from "./i18n";
import { platform } from "./platform";
import { saidPlainly } from "./refusal";

export type Association = { scheme: string; held: boolean };
export type Health =
  | "fine"
  | "stale"
  | "build_tree"
  | "wrong_binary"
  | "no_resident"
  | "mounted"
  | "not_registered";
export type SystemState = {
  registered: boolean;
  is_default: boolean;
  associations: Association[];
  starts_with_system: boolean;
  startup_is_ours: boolean;
  health: Health;
  registered_path: string | null;
  edge_installed: boolean;
};

/// Windows only lets the person pick it in Settings; macOS asks in its own prompt.
export async function chooseUs(state: SystemState | null): Promise<SystemState> {
  if (platform() === "macos") {
    return invoke<SystemState>("system_set_registered", { enabled: true });
  }
  const known = state ?? (await invoke<SystemState>("system_state"));
  const listed = known.registered
    ? known
    : await invoke<SystemState>("system_set_registered", { enabled: true });
  await invoke("system_open_default_apps");
  return listed;
}

export function useChooseUs(onChosen: (next: SystemState) => void) {
  const language = useSpoken();
  const [problem, setProblem] = useState<string | null>(null);
  const choose = useCallback(
    (state: SystemState | null) => {
      setProblem(null);
      chooseUs(state)
        .then(onChosen)
        .catch((e: unknown) => setProblem(saidPlainly(language, e)));
    },
    [language, onChosen],
  );
  return { choose, problem };
}
