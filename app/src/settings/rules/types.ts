import type { Key } from "../../i18n";

/// A browser with profiles is one destination per profile, as the picker offers them.
export type Destination = {
  browserId: string;
  profileId: string | null;
  label: string;
  icon: string | null;
  opensPrivately: boolean;
};

export type BrowserView = {
  id: string;
  name: string;
  profile_names: [string, string][];
  private: boolean;
  hidden: boolean;
  icon: string | null;
};

export type RuleView = {
  id: string;
  browser_id: string;
  profile_id: string | null;
  kind: "any" | "url" | "host" | "site";
  covers: string;
  browser: string;
  profile: string | null;
  icon: string | null;
  private: boolean;
  source_app: string | null;
  resolved: boolean;
  profile_gone: boolean;
};

export type Why = "alone" | "origin" | "narrower" | "earlier";

export type Probe = {
  winner: RuleView | null;
  why: Why | null;
  also: RuleView[];
  site: string;
};

/// «From» whenever an app is named, whatever else the rule covers.
export type Family = "app" | "url" | "host" | "site";

export function familyOf(rule: RuleView): Family {
  if (rule.source_app) return "app";
  return rule.kind === "any" ? "site" : rule.kind;
}

export const FAMILY_LABEL: Record<Family, Key> = {
  app: "kindFrom",
  url: "kindUrl",
  host: "kindHost",
  site: "kindSite",
};

export function covers(rule: RuleView): string {
  if (rule.source_app && rule.kind === "any") return rule.source_app;
  return rule.covers;
}

/// NUL separates the pair because no browser id or profile id can contain one.
const SPLIT = "\0";

export function named(browserId: string, profileId: string | null): string {
  return `${browserId}${SPLIT}${profileId ?? ""}`;
}

export function unnamed(key: string): { browserId: string; profileId: string | null } {
  const [browserId, profileId] = key.split(SPLIT);
  return { browserId, profileId: profileId || null };
}

export function destinationsOf(all: BrowserView[]): Destination[] {
  return all
    .filter((b) => !b.hidden)
    .flatMap<Destination>((b) =>
      b.profile_names.length > 0
        ? b.profile_names.map(([id, name]) => ({
            browserId: b.id,
            profileId: id,
            label: `${b.name} · ${name}`,
            icon: b.icon,
            opensPrivately: b.private,
          }))
        : [
            {
              browserId: b.id,
              profileId: null,
              label: b.name,
              icon: b.icon,
              opensPrivately: b.private,
            },
          ],
    );
}
