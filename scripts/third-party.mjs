import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const out = join(root, "THIRD-PARTY-BUNDLED.md");
const texts = join(root, "THIRD-PARTY-LICENSES.md");

const bundled = () => {
  const away = mkdtempSync(join(tmpdir(), "linkunbound-notices-"));
  try {
    execFileSync(
      process.execPath,
      [
        join(root, "app", "node_modules", "vite", "bin", "vite.js"),
        "build",
        "--sourcemap",
        "--emptyOutDir",
        "--logLevel",
        "error",
        "--outDir",
        away,
      ],
      { cwd: join(root, "app"), stdio: ["ignore", "ignore", "inherit"] },
    );
    const names = new Set();
    for (const one of readdirSync(join(away, "assets"))) {
      if (!one.endsWith(".map")) continue;
      const map = JSON.parse(readFileSync(join(away, "assets", one), "utf8"));
      for (const source of map.sources ?? []) {
        const where = source.replace(/\\/g, "/");
        const at = where.split("node_modules/").at(-1);
        if (at === where) continue;
        const parts = at.split("/");
        names.add(parts[0].startsWith("@") ? `${parts[0]}/${parts[1]}` : parts[0]);
      }
    }
    if (names.size === 0) throw new Error("the window's bundle named no package");
    return names;
  } finally {
    rmSync(away, { recursive: true, force: true });
  }
};

const shipped = () => {
  const lock = JSON.parse(readFileSync(join(root, "app", "package-lock.json"), "utf8"));
  const inside = bundled();
  const seen = new Map();
  for (const [at, one] of Object.entries(lock.packages ?? {})) {
    if (!at || one.dev || one.devOptional || one.extraneous) continue;
    const name = one.name ?? at.slice(at.lastIndexOf("node_modules/") + 13);
    if (!name || seen.has(name) || !inside.has(name)) continue;
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

const BSD3 = (who) => `BSD 3-Clause License

Copyright (c) ${who}

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its
   contributors may be used to endorse or promote products derived from
   this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.`;

const STANDARD = { MIT, ISC };
const HELD = { MIT, ISC, "BSD-3-Clause": BSD3 };
const CANONICAL = ["Apache-2.0", "BSL-1.0", "MPL-2.0"];

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

// What ships is the union of every build, so the list is the same whichever machine writes it.
const SHIPPED = ["x86_64-pc-windows-msvc", "aarch64-apple-darwin", "x86_64-apple-darwin"];

const crates = () => {
  const seen = new Map();
  for (const triple of SHIPPED) {
    const said = execFileSync(
      "cargo",
      [
        "tree",
        "--locked",
        "--workspace",
        "--edges",
        "normal,no-proc-macro",
        "--target",
        triple,
        "--prefix",
        "none",
        "--format",
        "{p}|{l}",
      ],
      { cwd: root, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
    );
    for (const line of said.split(/\r?\n/)) {
      const one = /^(\S+) v(\S+)(?: \((.+?)\))?\|(.*?)(?: \(\*\))?$/.exec(line);
      if (!one) continue;
      const [, name, version, from, licence] = one;
      const key = `${name}@${version}`;
      if ((from && existsSync(from)) || seen.has(key)) continue;
      seen.set(key, { name, version, licence: licence || "see the crate" });
    }
  }
  return seen;
};

const canonical = (licence) =>
  readFileSync(join(root, "scripts", "licences", `${licence}.txt`), "utf8");

const offered = (expression) =>
  expression
    .split(/\s+OR\s+|\//)
    .map((one) => one.replace(/[()]/g, "").trim())
    .filter(Boolean);

const declared = (pkg) => {
  if (/\s(AND|WITH)\s/.test(pkg.license ?? "")) return null;
  const choices = offered(pkg.license ?? "");
  const pick = CANONICAL.find((one) => choices.includes(one)) ?? choices.find((one) => one in HELD);
  if (!pick) return null;
  if (CANONICAL.includes(pick)) return canonical(pick);
  const who = pkg.authors?.length
    ? pkg.authors.map((one) => one.replace(/\s*<[^>]*>\s*/g, "").trim()).join(", ")
    : `the ${pkg.name} authors (${pkg.repository ?? `https://crates.io/crates/${pkg.name}`})`;
  return HELD[pick](who);
};

const carried = (pkg) => {
  const at = dirname(pkg.manifest_path);
  const files = new Set();
  for (const one of readdirSync(at)) {
    if (/^(licen[cs]e|copying|notice)/i.test(one) && statSync(join(at, one)).isFile()) {
      files.add(join(at, one));
    }
  }
  const reuse = join(at, "LICENSES");
  if (existsSync(reuse) && statSync(reuse).isDirectory()) {
    for (const one of readdirSync(reuse)) {
      if (statSync(join(reuse, one)).isFile()) files.add(join(reuse, one));
    }
  }
  if (pkg.license_file && existsSync(join(at, pkg.license_file))) {
    files.add(join(at, pkg.license_file));
  }
  return [...files].sort().map((one) => readFileSync(one, "utf8"));
};

const written = (rs) => {
  execFileSync("cargo", ["fetch", "--locked"], { cwd: root, stdio: "ignore" });
  const meta = JSON.parse(
    execFileSync("cargo", ["metadata", "--locked", "--format-version", "1"], {
      cwd: root,
      encoding: "utf8",
      maxBuffer: 128 * 1024 * 1024,
    }),
  );
  const packages = new Map(meta.packages.map((one) => [`${one.name}@${one.version}`, one]));
  const byText = new Map();
  const silent = [];
  for (const [key, crate] of rs) {
    const pkg = packages.get(key);
    const found = pkg ? carried(pkg) : [];
    if (found.length === 0) {
      const drafted = pkg && declared(pkg);
      if (!drafted) {
        silent.push(`${key} (${crate.licence})`);
        continue;
      }
      found.push(drafted);
    }
    for (const text of found) {
      const said = text.replace(/\r\n/g, "\n").trim();
      const sum = createHash("sha256").update(said).digest("hex");
      if (!byText.has(sum)) byText.set(sum, { said, crates: [] });
      byText.get(sum).crates.push(crate);
    }
  }
  if (silent.length > 0) {
    throw new Error(`no licence text for ${silent.length} crates:\n${silent.join("\n")}`);
  }
  return [...byText.values()]
    .map((one) => ({
      ...one,
      crates: one.crates.sort((a, b) => `${a.name}@${a.version}`.localeCompare(`${b.name}@${b.version}`)),
    }))
    .sort((a, b) =>
      `${a.crates[0].name}@${a.crates[0].version}`.localeCompare(
        `${b.crates[0].name}@${b.crates[0].version}`,
      ) || Number(a.said > b.said) - Number(a.said < b.said),
    );
};

const fenced = (said) => {
  const longest = Math.max(0, ...(said.match(/`+/g) ?? []).map((run) => run.length));
  const fence = "`".repeat(Math.max(3, longest + 1));
  return `${fence}text\n${said}\n${fence}`;
};

const section = (one, at) => {
  const who = one.crates.map((crate) => `\`${crate.name}\` ${crate.version}`).join(", ");
  return `## Text ${at + 1}\n\n${who}\n\n${fenced(one.said)}`;
};

const listed = (seen) =>
  [...seen.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([key, one]) => `| \`${one.name ?? key}\` | ${one.version} | ${one.licence} |`)
    .join("\n");

const js = shipped();
const rs = crates();
const licences = written(rs);

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
licence. Nothing of it was copied into LinkUnbound's own source. The licence text
of every crate is in [THIRD-PARTY-LICENSES.md](https://github.com/rgdevment/LinkUnbound/blob/main/THIRD-PARTY-LICENSES.md),
also under About → Licence texts.

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

writeFileSync(
  texts,
  asWritten(`# Licence texts — what the crates inside LinkUnbound carry

<!-- Written by \`npm run notices\`. Do not edit by hand. -->

Every crate in [THIRD-PARTY-BUNDLED.md](https://github.com/rgdevment/LinkUnbound/blob/main/THIRD-PARTY-BUNDLED.md)
comes with the licence text it ships, or, when it ships none, the text of the licence it declares.
Each text appears once, under the crates that carry it: ${licences.length} texts for ${rs.size} crates.

${licences.map(section).join("\n\n")}
`),
);

console.log(`${js.size} packages, ${rs.size} crates -> ${out}`);
console.log(`${licences.length} licence texts -> ${texts}`);
