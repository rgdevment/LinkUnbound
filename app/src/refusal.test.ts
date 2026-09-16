import { describe, expect, it } from "vitest";
import { SPEECH } from "./i18n";
import { saidPlainly } from "./refusal";

describe("refusals", () => {
  it("says a known refusal in the reader's language", () => {
    expect(saidPlainly("es", "updateStopped")).toBe("La actualización se detuvo");
    expect(saidPlainly("en", "updateStopped")).toBe("The update was stopped");
  });

  it("survives the Error wrapper the bridge puts around a rejection", () => {
    expect(saidPlainly("es", new Error("updateGone"))).toBe(
      "Ya no hay ninguna versión nueva que instalar",
    );
  });

  /// A Spanish window showing English prose from Rust — or the Debug of a Windows enum — is what
  /// this exists to prevent. The detail is kept, because it is the only thing that identifies the
  /// fault, but it arrives inside a sentence.
  it("never hands over a bare Rust error", () => {
    const said = saidPlainly("es", "error sending request for url (https://example.invalid)");

    expect(said).toContain("Algo salió mal");
    expect(said).toContain("example.invalid");
  });

  it("says something even when the fault carried no words", () => {
    expect(saidPlainly("es", "")).toBe("Algo salió mal");
  });

  /// The catalogue is a list written by hand, and the backend grows without asking it. This walks
  /// the Rust looking for what it can send, so a refusal added without a sentence fails here
  /// rather than reaching somebody as a camelCase identifier in a red box.
  ///
  /// Every literal is read, not only the ones already shaped like a key: a refusal written as a
  /// finished sentence in Rust is the worse case, because it reaches the reader looking like
  /// prose while being in whichever language the author happened to be writing in.
  it("has a sentence for every refusal the core can send", () => {
    const rust = import.meta.glob("../src-tauri/src/**/*.rs", {
      eager: true,
      query: "?raw",
      import: "default",
    }) as Record<string, string>;
    const said = new Set<string>();

    for (const body of Object.values(rust)) {
      for (const [, key] of body.matchAll(
        /(?:Err|ok_or_else)\(\s*(?:\|\|\s*)?"([^"]+)"\.to_owned\(\)\s*\)/g,
      )) {
        said.add(key);
      }
    }

    expect(Object.keys(rust).length, "no Rust was read at all").toBeGreaterThan(3);
    expect(said.size, "no refusals were found; the pattern stopped matching").toBeGreaterThan(3);
    for (const key of said) {
      expect(
        SPEECH.es,
        `the core can send «${key}» and the window has no sentence for it`,
      ).toHaveProperty(key);
      expect(SPEECH.en, `«${key}» has a sentence in Spanish and none in English`).toHaveProperty(
        key,
      );
    }
  });
});

/// Every key the Rust side can answer with has to have a sentence here, or the window shows
/// "Algo salió mal — notBundled" and nobody notices until a person does.
describe("what the Mac backend can refuse with", () => {
  const sources = import.meta.glob(
    [
      "../../crates/linkunbound-mac/src/registration.rs",
      "../../crates/linkunbound-mac/src/startup.rs",
      "../src-tauri/src/system.rs",
    ],
    { query: "?raw", import: "default", eager: true },
  ) as Record<string, string>;

  it("is spoken in both languages", () => {
    const keys = new Set<string>();
    for (const text of Object.values(sources)) {
      for (const found of text.matchAll(
        /Err\("([a-zA-Z]+)"\.to_owned\(\)\)|\|\| "([a-zA-Z]+)"\.to_owned\(\)/g,
      )) {
        keys.add(found[1] ?? found[2]);
      }
    }
    expect(Object.keys(sources)).toHaveLength(3);
    expect(keys.size).toBeGreaterThanOrEqual(5);
    for (const key of keys) {
      expect(SPEECH.es, key).toHaveProperty(key);
      expect(SPEECH.en, key).toHaveProperty(key);
    }
  });
});
