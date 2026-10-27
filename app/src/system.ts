import { invoke } from "@tauri-apps/api/core";
import { platform } from "./platform";

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

/// macOS asks the person in its own prompt; Windows only lets them pick it in Settings, so the
/// most this can do there is put LinkUnbound in the list and open its page.
export async function chooseUs(state: SystemState): Promise<SystemState> {
  if (platform() === "macos") {
    return invoke<SystemState>("system_set_registered", { enabled: true });
  }
  const listed = state.registered
    ? state
    : await invoke<SystemState>("system_set_registered", { enabled: true });
  await invoke("system_open_default_apps");
  return listed;
}
