import type { Language } from "../i18n";
import news from "../news.json";

export type Told = { title: string; said: string };

export function toldFor(
  versions: string[],
  language: Language,
): { version: string; told: Told[] }[] {
  return versions.flatMap((version) => {
    const found = news.find((one) => one.version === version);
    return found ? [{ version, told: found[language] }] : [];
  });
}

export function headline(version: string | undefined, language: Language): string | undefined {
  return news.find((one) => one.version === version)?.headline[language];
}
