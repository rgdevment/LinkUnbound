import { describe, expect, it } from "vitest";

const css = Object.values(
  import.meta.glob<string>("./index.css", { query: "?raw", import: "default", eager: true }),
)[0];

function palette(...blocks: RegExp[]): Record<string, string> {
  const vars: Record<string, string> = {};
  for (const block of blocks) {
    for (const [, body] of css.matchAll(block)) {
      for (const [, name, value] of body.matchAll(/--([\w-]+):\s*(#[0-9a-fA-F]{6})\s*;/g)) {
        vars[name] = value;
      }
    }
  }
  return vars;
}

function luminance(hex: string) {
  const [r, g, b] = [1, 3, 5]
    .map((i) => Number.parseInt(hex.slice(i, i + 2), 16) / 255)
    .map((c) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4));
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function ratio(a: string, b: string) {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

const light = palette(/^:root\s*\{([^}]*)\}/gm, /^:root\.contrast\s*\{([^}]*)\}/gm);
const dark = palette(
  /^:root\s*\{([^}]*)\}/gm,
  /^\.dark\s*\{([^}]*)\}/gm,
  /^:root\.dark\.contrast\s*\{([^}]*)\}/gm,
);

describe.each([
  ["light", light],
  ["dark", dark],
])("high contrast on the %s theme", (_name, colours) => {
  it.each(["muted", "ink-2"])("%s reads at 4.5:1 on the panel and the card", (text) => {
    for (const ground of ["panel", "card"]) {
      expect(ratio(colours[text], colours[ground]), `${text} on ${ground}`).toBeGreaterThanOrEqual(
        4.5,
      );
    }
  });
});

describe("the high contrast palettes", () => {
  const base = palette(/^:root\s*\{([^}]*)\}/gm);
  const baseDark = palette(/^\.dark\s*\{([^}]*)\}/gm);

  it("are really there for both themes", () => {
    expect(css).toMatch(/:root\.contrast\s*\{/);
    expect(css).toMatch(/:root\.dark\.contrast\s*\{/);
    expect(light.muted).not.toBe(base.muted);
    expect(dark.muted).not.toBe(baseDark.muted);
  });

  it("never read worse than the normal ones", () => {
    expect(ratio(light.muted, light.panel)).toBeGreaterThanOrEqual(ratio(base.muted, base.panel));
    expect(ratio(dark.muted, dark.panel)).toBeGreaterThanOrEqual(
      ratio(baseDark.muted, baseDark.panel),
    );
  });
});

describe("the edge token", () => {
  const tokens = (...blocks: RegExp[]) => {
    const vars: Record<string, string> = {};
    for (const block of blocks) {
      for (const [, body] of css.matchAll(block)) {
        for (const [, name, value] of body.matchAll(/--(hair|edge):\s*(rgb\([^)]*\))\s*;/g)) {
          vars[name] = value;
        }
      }
    }
    return vars;
  };
  const alpha = (rgb: string) => Number(rgb.match(/\/\s*([\d.]+)\s*\)/)?.[1]);

  const normal = tokens(/^:root\s*\{([^}]*)\}/gm);
  const normalDark = tokens(/^\.dark\s*\{([^}]*)\}/gm);
  const strong = tokens(/^:root\.contrast\s*\{([^}]*)\}/gm);
  const strongDark = tokens(/^:root\.dark\.contrast\s*\{([^}]*)\}/gm);

  it("is defined for all four palettes, black on light and white on dark", () => {
    expect(normal.edge).toBe("rgb(0 0 0 / 0.12)");
    expect(normalDark.edge).toBe("rgb(255 255 255 / 0.12)");
    expect(strong.edge).toBe("rgb(0 0 0 / 0.32)");
    expect(strongDark.edge).toBe("rgb(255 255 255 / 0.32)");
  });

  it.each([
    ["light", normal, strong],
    ["dark", normalDark, strongDark],
  ])("turns more opaque with high contrast, with the hairline too (%s)", (_n, base, high) => {
    expect(alpha(high.edge)).toBeGreaterThan(alpha(base.edge));
    expect(alpha(high.hair)).toBeGreaterThan(alpha(base.hair));
  });

  it("is offered to the utilities as a colour", () => {
    expect(css).toMatch(/--color-edge:\s*var\(--edge\)/);
  });
});
