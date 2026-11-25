import { describe, expect, it } from "vitest";

const css = Object.values(
  import.meta.glob<string>("./index.css", { query: "?raw", import: "default", eager: true }),
)[0];

function palette(block: RegExp): Record<string, string> {
  const blocks = [...css.matchAll(block)];
  const vars: Record<string, string> = {};
  for (const [, body] of blocks) {
    for (const [, name, value] of body.matchAll(/--([\w-]+):\s*(#[0-9a-fA-F]{6})\s*;/g)) {
      vars[name] = value;
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

function blend(top: string, alpha: number, ground: string) {
  const channel = (hex: string, i: number) => Number.parseInt(hex.slice(i, i + 2), 16);
  return `#${[1, 3, 5]
    .map((i) => Math.round(channel(top, i) * alpha + channel(ground, i) * (1 - alpha)))
    .map((c) => c.toString(16).padStart(2, "0"))
    .join("")}`;
}

const rootBlock = /^:root\s*\{([^}]*)\}/gm;
const darkBlock = /^\.dark\s*\{([^}]*)\}/gm;
const themes = { light: palette(rootBlock), dark: palette(darkBlock) };

describe.each(Object.entries(themes))("%s theme contrast", (name, colours) => {
  it.each(["muted", "ink-2", "caution"])("%s text reads on the panel and the card", (text) => {
    for (const ground of ["panel", "card"]) {
      expect(ratio(colours[text], colours[ground]), `${text} on ${ground}`).toBeGreaterThanOrEqual(
        4.5,
      );
    }
  });

  it("keeps secondary and tinted text readable on the tinted grounds it sits on", () => {
    const overlay = name === "light" ? "#000000" : "#ffffff";
    const chip = blend(overlay, name === "light" ? 0.045 : 0.06, colours.panel);
    const grounds: [string, string, string][] = [
      ["muted on a chip", colours.muted, chip],
      ["ink-2 on a chip", colours["ink-2"], chip],
      ["caution in its box", colours.caution, blend(colours.caution, 0.07, colours.panel)],
      [
        "accent on its pill",
        colours.accent,
        blend(colours.accent, name === "light" ? 0.13 : 0.17, colours.panel),
      ],
    ];
    for (const [what, text, ground] of grounds) {
      expect(ratio(text, ground), what).toBeGreaterThanOrEqual(4.5);
    }
  });

  it("keeps the main ink readable on the panel and the card", () => {
    for (const ground of ["panel", "card"]) {
      expect(ratio(colours.ink, colours[ground])).toBeGreaterThanOrEqual(4.5);
    }
  });

  it.each(["accent", "danger", "caution"])("on-accent text reads over %s", (ground) => {
    expect(
      ratio(colours["on-accent"], colours[ground]),
      `on-accent on ${ground}`,
    ).toBeGreaterThanOrEqual(4.5);
  });
});
