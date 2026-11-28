// What the lockfiles let in: every package from the npm registry with its sha512, and none that
// runs code when it is installed except the ones named here and why. CI installs with
// --ignore-scripts, so a new install script would break quietly or, run by hand, run unread.
import { readFileSync } from "node:fs";

const LOCKFILES = ["package-lock.json", "app/package-lock.json"];
const REGISTRY = "https://registry.npmjs.org/";

// Vite's optional file watcher on macOS; it ships its binary prebuilt and is never needed to build.
const MAY_RUN_AT_INSTALL = new Set(["fsevents"]);

const nameOf = (path) => path.slice(path.lastIndexOf("node_modules/") + "node_modules/".length);

function problemsIn(lockfile) {
  const lock = JSON.parse(readFileSync(lockfile, "utf8"));
  if (lock.lockfileVersion !== 3) {
    return [`${lockfile} is lockfileVersion ${lock.lockfileVersion}, and this reads 3`];
  }
  const found = [];
  for (const [path, entry] of Object.entries(lock.packages ?? {})) {
    if (path === "" || entry.link) continue;
    const name = nameOf(path);
    if (entry.hasInstallScript && !MAY_RUN_AT_INSTALL.has(name)) {
      found.push(`${lockfile}: ${name} ${entry.version} runs a script when it is installed`);
    }
    // A package bundled inside another rides on that one's hash and has no address of its own.
    if (entry.inBundle) continue;
    if (!entry.resolved?.startsWith(REGISTRY)) {
      found.push(`${lockfile}: ${name} comes from ${entry.resolved ?? "nowhere named"}, not the npm registry`);
    }
    if (!entry.integrity?.startsWith("sha512-")) {
      found.push(`${lockfile}: ${name} ${entry.version} carries no sha512 to check it against`);
    }
  }
  return found;
}

const problems = LOCKFILES.flatMap(problemsIn);
for (const said of problems) {
  console.log(process.env.GITHUB_ACTIONS ? `::error::${said}` : `x  ${said}`);
}
if (problems.length > 0) process.exit(1);
console.log(`ok ${LOCKFILES.join(" and ")}: registry packages only, no new install scripts`);
