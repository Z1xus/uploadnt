import { createHash } from "node:crypto";
import { writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const [site = "https://uploadnt.com", repo] = process.argv.slice(2);
const sha = (data: ArrayBuffer | string) =>
  createHash("sha256")
    .update(typeof data === "string" ? data : new Uint8Array(data))
    .digest("hex");

const root = await fetch(site);
const claimed = await (
  await fetch(new URL("/.well-known/provenance.json", site))
).json();
const listed = await (
  await fetch(new URL("/.well-known/bundle.txt", site))
).text();

const lines = await Promise.all(
  listed
    .trim()
    .split("\n")
    .map(async (line) => {
      const file = line.slice(66);
      const res = await fetch(
        new URL("/" + file.replace(/(^|\/)index\.html$/, "$1"), site),
      );
      return `${sha(await res.arrayBuffer())}  ${file}\n`;
    }),
);
const served = lines.join("");
const bundle = sha(served);

const problems = [
  served !== listed && "served files differ from the listed bundle",
  bundle !== claimed.bundle && "bundle hash differs from provenance.json",
  root.headers.get("x-uploadnt-bundle") !== bundle &&
    "bundle header differs from served files",
  root.headers.get("x-uploadnt-commit") !== claimed.commit &&
    "commit header differs from provenance.json",
].filter(Boolean);

if (repo && !problems.length) {
  const path = join(tmpdir(), `uploadnt-bundle-${bundle}.txt`);
  writeFileSync(path, served);
  const gh = Bun.spawnSync(
    [
      "gh",
      "attestation",
      "verify",
      path,
      "--repo",
      repo,
      "--source-digest",
      claimed.commit,
    ],
    { stderr: "inherit" },
  );
  if (gh.exitCode !== 0) problems.push("no github attestation for this bundle");
}

console.log(
  JSON.stringify(
    { site, commit: claimed.commit, bundle, ok: !problems.length, problems },
    null,
    2,
  ),
);
process.exit(problems.length ? 1 : 0);
