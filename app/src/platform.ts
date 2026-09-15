export type Platform = "windows" | "macos";

export function platform(): Platform {
  return /mac/i.test(globalThis.navigator?.platform ?? "") ? "macos" : "windows";
}
