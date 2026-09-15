import { fill, type Key, type Language, SPEECH, said } from "./i18n";

/// Every refusal the backend can send travels as a catalogue key. A fault nobody wrote a sentence
/// for still has to say something in the reader's language, or a Spanish window shows English
/// prose from Rust — or a `Debug` of a Windows enum, which is worse.
export function saidPlainly(language: Language, problem: unknown): string {
  const raw = String(problem)
    .replace(/^Error:\s*/, "")
    .trim();
  const words = SPEECH[language];

  if (raw in words) {
    return said(language, raw as Key);
  }
  return raw
    ? fill(said(language, "internal"), raw)
    : said(language, "internal").replace(" — {}", "");
}
