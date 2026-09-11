import { describe, expect, it } from "vitest";
import { fill, type Key, SPEECH, spoken } from "./i18n";

const KEYS = Object.keys(SPEECH.es) as Key[];

describe("catalogue", () => {
  it("says the same things in both languages", () => {
    expect(Object.keys(SPEECH.en).sort()).toEqual(KEYS.slice().sort());
  });

  it("ships nothing blank", () => {
    for (const key of KEYS) {
      expect(SPEECH.es[key].trim(), `${key} in Spanish`).not.toBe("");
      expect(SPEECH.en[key].trim(), `${key} in English`).not.toBe("");
    }
  });

  it("keeps every placeholder in both languages", () => {
    for (const key of KEYS) {
      const es = SPEECH.es[key].match(/\{\}/g)?.length ?? 0;
      const en = SPEECH.en[key].match(/\{\}/g)?.length ?? 0;
      expect(en, `${key} disagrees about its placeholders`).toBe(es);
    }
  });

  it("fills the markers in the order they are given", () => {
    expect(fill(SPEECH.es.associations, "2", "3")).toBe("2 de 3 asociaciones");
    expect(fill(SPEECH.en.associations, "2", "3")).toBe("2 of 3 associations");
  });

  it("only takes english when the backend actually said so", () => {
    expect(spoken("en")).toBe("en");
    expect(spoken("es")).toBe("es");
    expect(spoken(undefined)).toBe("es");
  });
});
