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
  it("has a sentence for every refusal the core can send", () => {
    const rust = import.meta.glob("../src-tauri/src/**/*.rs", {
      eager: true,
      query: "?raw",
      import: "default",
    }) as Record<string, string>;
    const said = new Set<string>();

    for (const body of Object.values(rust)) {
      for (const [, key] of body.matchAll(
        /(?:Err|ok_or_else)\(\s*(?:\|\|\s*)?"([a-z][A-Za-z]+)"\.to_owned\(\)\s*\)/g,
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
    }
  });
});
