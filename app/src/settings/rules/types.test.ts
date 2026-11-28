import { describe, expect, it } from "vitest";
import {
  type BrowserView,
  covers,
  destinationsOf,
  familyOf,
  named,
  type RuleView,
  unnamed,
} from "./types";

const rule = (over: Partial<RuleView>): RuleView => ({
  id: "r",
  browser_id: "chrome",
  profile_id: null,
  kind: "site",
  covers: "github.com",
  browser: "Chrome",
  profile: null,
  icon: null,
  private: false,
  source_app: null,
  resolved: true,
  profile_gone: false,
  ...over,
});

describe("rule kinds", () => {
  it("files a rule under «From» whenever an app is named, whatever else it covers", () => {
    expect(familyOf(rule({ kind: "any", covers: "", source_app: "teams" }))).toBe("app");
    expect(familyOf(rule({ kind: "site", source_app: "teams" }))).toBe("app");
  });

  it("files the others under the kind of address they cover", () => {
    expect(familyOf(rule({ kind: "url" }))).toBe("url");
    expect(familyOf(rule({ kind: "host" }))).toBe("host");
    expect(familyOf(rule({ kind: "site" }))).toBe("site");
    expect(familyOf(rule({ kind: "any" }))).toBe("site");
    expect(familyOf(rule({ kind: "file_type", covers: "pdf" }))).toBe("file_type");
  });

  it("reads a rule for an app as the app, and any other as what it covers", () => {
    expect(covers(rule({ kind: "any", covers: "", source_app: "teams" }))).toBe("teams");
    expect(covers(rule({ kind: "site", source_app: "teams" }))).toBe("github.com");
    expect(covers(rule({}))).toBe("github.com");
    expect(covers(rule({ kind: "file_type", covers: "pdf" }))).toBe(".pdf");
  });
});

describe("destinations", () => {
  it("travel as one string that gives back the browser and the profile", () => {
    expect(unnamed(named("chrome", "Profile 2"))).toEqual({
      browserId: "chrome",
      profileId: "Profile 2",
    });
    expect(unnamed(named("firefox", null))).toEqual({ browserId: "firefox", profileId: null });
  });

  const view = (over: Partial<BrowserView>): BrowserView => ({
    id: "b",
    name: "B",
    profile_names: [],
    private: true,
    hidden: false,
    icon: null,
    ...over,
  });

  it("are one per profile, and one for a browser without profiles", () => {
    const all = destinationsOf([
      view({
        id: "chrome",
        name: "Chrome",
        profile_names: [
          ["Default", "Casa"],
          ["P2", "Trabajo"],
        ],
        icon: "data:x",
      }),
      view({ id: "plain", name: "Plain", private: false }),
    ]);
    expect(all).toEqual([
      {
        browserId: "chrome",
        profileId: "Default",
        label: "Chrome · Casa",
        icon: "data:x",
        opensPrivately: true,
      },
      {
        browserId: "chrome",
        profileId: "P2",
        label: "Chrome · Trabajo",
        icon: "data:x",
        opensPrivately: true,
      },
      { browserId: "plain", profileId: null, label: "Plain", icon: null, opensPrivately: false },
    ]);
  });

  it("leave out the browsers the person hid", () => {
    expect(destinationsOf([view({ hidden: true })])).toEqual([]);
  });
});
