import { fill, type Key, type Language, SPEECH } from "./i18n";

/// Every refusal the backend can send travels as a catalogue key. A fault nobody wrote a sentence
/// for still has to say something in the reader's language, or a Spanish window shows English
/// prose from Rust — or a `Debug` of a Windows enum, which is worse.
export function saidPlainly(language: Language, problem: unknown): string {
  const raw = String(problem)
    .replace(/^Error:\s*/, "")
    .trim();
  const words = SPEECH[language];

  if (raw in words) {
    return words[raw as Key];
  }
  return raw ? fill(words.internal, raw) : words.internal.replace(" — {}", "");
}
