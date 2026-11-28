import { describe, expect, it } from "vitest";
import bundled from "../../THIRD-PARTY-BUNDLED.md?raw";
import licences from "../../THIRD-PARTY-LICENSES.md?raw";
import { composed } from "./markdown";

const drawn = (text: string) => {
  const holder = document.createElement("div");
  holder.append(composed(text));
  return holder;
};

describe("the notices the binary carries", () => {
  for (const [name, text] of [
    ["THIRD-PARTY-BUNDLED.md", bundled],
    ["THIRD-PARTY-LICENSES.md", licences],
  ]) {
    it(`${name} links only to places outside the window`, () => {
      const links = [...drawn(text).querySelectorAll("a")].map((one) => one.getAttribute("href"));
      expect(links.length).toBeGreaterThan(0);
      expect(links.filter((href) => !/^(https?:\/\/|mailto:)/.test(href ?? ""))).toEqual([]);
    });

    it(`${name} draws its headings, its tables and its licence texts`, () => {
      const page = drawn(text);
      expect(page.querySelector("h1")?.textContent).toMatch(/LinkUnbound/);
      expect(page.querySelectorAll("pre code").length).toBeGreaterThan(0);
      expect(page.textContent).not.toContain("<!--");
    });
  }
});

describe("what the notices are drawn from", () => {
  it("never turns a licence's words into markup", () => {
    const page = drawn(
      [
        "<img src=x onerror=alert(1)> and <b>bold</b>",
        "",
        "```text",
        "<script>alert(1)</script>",
        "[a link](https://a.test/) **strong**",
        "```",
      ].join("\n"),
    );
    expect(page.querySelector("img, b, script, strong")).toBeNull();
    expect(page.querySelector("p")?.textContent).toBe(
      "<img src=x onerror=alert(1)> and <b>bold</b>",
    );
    expect(page.querySelector("pre a")).toBeNull();
    expect(page.querySelector("pre")?.textContent).toContain("<script>alert(1)</script>");
  });

  it("keeps a link's words and drops an address that leads anywhere but out", () => {
    const page = drawn("[run](javascript:alert(1)) [file](file:///C:/x) [site](https://a.test/x)");
    expect([...page.querySelectorAll("a")].map((one) => one.getAttribute("href"))).toEqual([
      "https://a.test/x",
    ]);
    expect(page.textContent).toBe("run file site");
  });

  it("makes a bare web address a link, short of the full stop that ends its sentence", () => {
    const page = drawn(
      "See https://crates.io/crates/slint. Or [`slint`](https://slint.dev/) itself.",
    );
    expect([...page.querySelectorAll("a")].map((one) => one.getAttribute("href"))).toEqual([
      "https://crates.io/crates/slint",
      "https://slint.dev/",
    ]);
    expect(page.querySelector("a a")).toBeNull();
    expect(page.textContent).toBe("See https://crates.io/crates/slint. Or slint itself.");
  });

  it("reads a table, with code in its cells, as a head and a body", () => {
    const page = drawn(
      "| Package | Version |\n| --- | --- |\n| `react` | 19.3.1 |\n| `scheduler` | 0.28.0 |",
    );
    expect([...page.querySelectorAll("thead th")].map((one) => one.textContent)).toEqual([
      "Package",
      "Version",
    ]);
    expect(page.querySelectorAll("tbody tr")).toHaveLength(2);
    expect(page.querySelector("tbody td code")?.textContent).toBe("react");
  });

  it("closes a fence only with one as long as the one that opened it", () => {
    const page = drawn("````text\nshown with ```\n```\nstill inside\n````\nafter");
    expect(page.querySelector("pre")?.textContent).toBe("shown with ```\n```\nstill inside\n");
    expect(page.querySelector("p")?.textContent).toBe("after");
  });

  it("joins the lines of a paragraph and parts paragraphs at a blank line", () => {
    const page = drawn("# Title with `code`\n\none\ntwo\n\nthree");
    expect(page.querySelector("h1 code")?.textContent).toBe("code");
    expect([...page.querySelectorAll("p")].map((one) => one.textContent)).toEqual([
      "one\ntwo",
      "three",
    ]);
  });
});
