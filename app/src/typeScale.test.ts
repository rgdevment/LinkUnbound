import { describe, expect, it } from "vitest";

const css = Object.values(
  import.meta.glob<string>("./index.css", { query: "?raw", import: "default", eager: true }),
)[0];

const modules = import.meta.glob<string>(["./**/*.tsx", "!./**/*.test.tsx"], {
  query: "?raw",
  import: "default",
  eager: true,
});
const files = Object.entries(modules).map(([path, text]) => ({ path: path.slice(2), text }));

function offenders(pattern: RegExp, allowed: string[] = []) {
  return files.flatMap(({ path, text }) =>
    (text.match(pattern) ?? [])
      .filter((hit) => !allowed.includes(hit))
      .map((hit) => `${path}: ${hit}`),
  );
}

describe("type scale and colour contract", () => {
  it("scans the settings sources", () => {
    expect(files.length).toBeGreaterThan(8);
  });

  it("uses no arbitrary pixel text sizes", () => {
    expect(offenders(/text-\[[\d.]+px\]/g)).toEqual([]);
  });

  it("uses no neutral palette text colours", () => {
    expect(offenders(/text-neutral-\d+/g)).toEqual([]);
  });

  it("puts no hex colour in a class name beyond the star icon", () => {
    const hexClass = /[\w:-]*-\[#[0-9A-Fa-f]{3,8}\]/g;
    expect(offenders(hexClass, ["text-[#C8892A]", "dark:text-[#E3B341]"])).toEqual([]);
  });

  it("keeps hex colours in markup to the decorative About fills", () => {
    const decorative = ["#C8892A", "#E3B341", "#e3b341", "#0078d4", "#db61a2", "#c8892a"];
    expect(offenders(/#[0-9A-Fa-f]{6}\b/g, decorative)).toEqual([]);
  });

  it("names themed colours by token, not by the raw variable", () => {
    const raw = /[\w:-]*-\(--(ink|ink-2|muted|accent|on-accent|danger|good|caution|panel|card)\)/g;
    expect(offenders(raw)).toEqual([]);
  });

  // Only the theme is held to five: a text-size class may still override them elsewhere.
  it("defines exactly the five text sizes in the theme", () => {
    const theme = css.match(/@theme\s*\{([^}]*)\}/)?.[1] ?? "";
    const sizes = [...theme.matchAll(/--text-(\w+):\s*([\d.]+)px/g)].map((m) => [m[1], m[2]]);
    expect(sizes).toEqual([
      ["xs", "11"],
      ["sm", "12"],
      ["md", "13"],
      ["lg", "15"],
      ["xl", "20"],
    ]);
    expect(theme).toMatch(/--text-\*:\s*initial/);
  });
});
