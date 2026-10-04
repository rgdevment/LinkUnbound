import { describe, expect, it } from "vitest";
import bundled from "../../THIRD-PARTY-BUNDLED.md?raw";
import licences from "../../THIRD-PARTY-LICENSES.md?raw";
import { composed } from "./markdown";

describe("the notices the binary carries", () => {
  for (const [name, text] of [
    ["THIRD-PARTY-BUNDLED.md", bundled],
    ["THIRD-PARTY-LICENSES.md", licences],
  ]) {
    it(`${name} links only to places outside the window`, () => {
      const page = new DOMParser().parseFromString(composed(text), "text/html");
      const links = [...page.querySelectorAll("a")].map((one) => one.getAttribute("href") ?? "");
      expect(links.filter((href) => !/^(https?:\/\/|mailto:)/.test(href))).toEqual([]);
    });
  }
});
