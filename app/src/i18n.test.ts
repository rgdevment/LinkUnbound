import { afterEach, describe, expect, it } from "vitest";
import { fill, type Key, MAC, SPEECH, said, spoken } from "./i18n";

const KEYS = Object.keys(SPEECH.es) as Key[];
const MAC_KEYS = Object.keys(MAC.es) as Key[];

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

  it("says the same things about a Mac in both languages", () => {
    const alphabetically = (a: string, b: string) => a.localeCompare(b);
    expect(Object.keys(MAC.en).sort(alphabetically)).toEqual(MAC_KEYS.slice().sort(alphabetically));
    for (const key of MAC_KEYS) {
      expect(KEYS).toContain(key);
      const es = MAC.es[key]?.match(/\{\}/g)?.length ?? 0;
      const en = MAC.en[key]?.match(/\{\}/g)?.length ?? 0;
      expect(en, `${key} disagrees about its placeholders`).toBe(es);
      expect(es, `${key} changes its placeholders on a Mac`).toBe(
        SPEECH.es[key].match(/\{\}/g)?.length ?? 0,
      );
    }
  });

  describe("on a Mac", () => {
    afterEach(() => {
      Object.defineProperty(navigator, "platform", { value: "", configurable: true });
    });

    it("says what a Mac would, and otherwise what everyone does", () => {
      expect(said("es", "defaultNo")).toContain("Windows");
      Object.defineProperty(navigator, "platform", { value: "MacIntel", configurable: true });
      expect(said("es", "defaultNo")).toContain("macOS");
      expect(said("en", "chooseGo")).toBe("Open System Settings");
      expect(said("es", "navLinks")).toBe(SPEECH.es.navLinks);
    });
  });

  it("only takes english when the backend actually said so", () => {
    expect(spoken("en")).toBe("en");
    expect(spoken("es")).toBe("es");
    expect(spoken(undefined)).toBe("es");
  });
});
