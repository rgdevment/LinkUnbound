import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const out = join(root, "THIRD-PARTY-BUNDLED.md");

const shipped = () => {
  const lock = JSON.parse(readFileSync(join(root, "app", "package-lock.json"), "utf8"));
  const seen = new Map();
  for (const [at, one] of Object.entries(lock.packages ?? {})) {
    if (!at || one.dev || one.devOptional || one.extraneous) continue;
    const name = one.name ?? at.slice(at.lastIndexOf("node_modules/") + 13);
    if (!name || seen.has(name)) continue;
    seen.set(name, {
      version: one.version ?? "?",
      licence: one.license ?? "see the package",
      notice: noticed(join(root, "app", at)),
    });
  }
  return seen;
};

const told = (pkg) =>
  typeof pkg.license === "string"
    ? pkg.license
    : (pkg.license?.type ?? pkg.licenses?.map((one) => one.type).join(" OR ") ?? "see the package");

const MIT = (who) => `MIT License

Copyright (c) ${who}

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.`;

const ISC = (who) => `ISC License

Copyright (c) ${who}

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE.`;

const STANDARD = { MIT, ISC };

const authored = (pkg) => {
  const who = typeof pkg.author === "string" ? pkg.author : pkg.author?.name;
  const named = who ?? pkg.contributors?.[0]?.name ?? pkg.maintainers?.[0]?.name;
  return named ? named.replace(/\s*<[^>]*>\s*/g, "").trim() : null;
};

const homed = (pkg) => {
  const at = pkg.repository?.url ?? pkg.repository ?? pkg.homepage;
  if (typeof at !== "string") return null;
  return at
    .replace(/^git\+/, "")
    .replace(/^git:\/\//, "https://")
    .replace(/^git@github\.com:/, "https://github.com/")
    .replace(/^git\+ssh:\/\/git@/, "https://")
    .replace(/\.git$/, "");
};

const drafted = (pkg, licence) => {
  const make = STANDARD[licence];
  if (!make) return null;
  const who = authored(pkg);
  const at = homed(pkg);
  const said = make(who ?? `the ${pkg.name} authors`);
  const from = at ? `\n\nThe package ships no licence file. Its text is at ${at}` : "";
  return `${said}${from}`;
};

const noticed = (at) => {
  if (!existsSync(at)) return null;
  const named = readdirSync(at).find((one) => /^(licen[cs]e|copying)/i.test(one));
  if (named) {
    const said = readFileSync(join(at, named), "utf8").trim();
    return said.length > 4000 ? `${said.slice(0, 4000)}\n…` : said;
  }
  const where = join(at, "package.json");
  if (!existsSync(where)) return null;
  const pkg = JSON.parse(readFileSync(where, "utf8"));
  return drafted(pkg, told(pkg));
};

const crates = () => {
  const said = execFileSync(
    "cargo",
    ["metadata", "--format-version", "1", "--filter-platform", process.env.TARGET ?? hostTriple()],
    { cwd: root, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  const meta = JSON.parse(said);
  const ours = new Set(meta.workspace_members);
  const seen = new Map();
  for (const one of meta.packages) {
    if (ours.has(one.id) || seen.has(one.name)) continue;
    seen.set(one.name, { version: one.version, licence: one.license ?? "see the crate" });
  }
  return seen;
};

const hostTriple = () => {
  const said = execFileSync("rustc", ["-vV"], { encoding: "utf8" });
  return /host: (.+)/.exec(said)?.[1]?.trim() ?? "";
};

const listed = (seen) =>
  [...seen.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([name, one]) => `| \`${name}\` | ${one.version} | ${one.licence} |`)
    .join("\n");

const js = shipped();
const rs = crates();

const kept = [...js.entries()]
  .filter(([, one]) => one.notice)
  .sort(([a], [b]) => a.localeCompare(b))
  .map(([name, one]) => `### \`${name}\` — ${one.licence}\n\n\`\`\`text\n${one.notice}\n\`\`\``)
  .join("\n\n");

const asWritten = (text) => text.replace(/\r\n/g, "\n");

writeFileSync(
  out,
  asWritten(`# Third-party notices — what ships inside LinkUnbound

<!-- Written by \`npm run notices\`. Do not edit by hand. -->

LinkUnbound is GPL-3.0-only. The binary carries the work below, each under its own
licence. Nothing of it was copied into LinkUnbound's own source.

## In the window (${js.size} packages)

| Package | Version | Licence |
| --- | --- | --- |
${listed(js)}

## In the core (${rs.size} crates)

| Crate | Version | Licence |
| --- | --- | --- |
${listed(rs)}

## The notices themselves

${kept}
`),
);

console.log(`${js.size} packages, ${rs.size} crates -> ${out}`);
