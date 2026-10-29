import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";

const dir = process.argv[2] ?? "web/build";
const commit = process.env.GITHUB_SHA ?? "local";
const stats = process.env.PUBLIC_IMPACTITE ?? "";
const digest = (algo: string, data: Buffer | string) =>
  createHash(algo).update(data).digest("base64");

const walk = (d: string): string[] =>
  readdirSync(d, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? walk(join(d, e.name)) : [join(d, e.name)],
  );

const html = walk(dir).filter((f) => f.endsWith(".html"));
const scripts = walk(dir).filter((f) => f.endsWith(".js"));
const integrity = Object.fromEntries(
  scripts.map((f) => [
    "/" + relative(dir, f),
    "sha384-" + digest("sha384", readFileSync(f)),
  ]),
);
const importmap = `<script type="importmap">${JSON.stringify({ integrity })}</script>`;

const inline = new Set<string>();
for (const f of html) {
  const page = readFileSync(f, "utf8")
    .replace("<head>", `<head>${importmap}`)
    .replace(
      /<link href="\.(\/_app\/[^"]+\.js)" rel="modulepreload">/g,
      (tag, path) => tag.replace(">", ` integrity="${integrity[path]}">`),
    );
  for (const [, body] of page.matchAll(
    /<script(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/g,
  )) {
    inline.add(`'sha256-${digest("sha256", body)}'`);
  }
  writeFileSync(f, page);
}

const csp = [
  "default-src 'self'",
  `script-src 'self' 'wasm-unsafe-eval' ${[...inline].join(" ")}`,
  "style-src 'self' 'unsafe-inline'",
  `connect-src 'self' wss: https://rtc.live.cloudflare.com ${stats}`,
  "img-src 'self' data: blob:",
  "worker-src 'self' blob:",
  "base-uri 'none'",
  "form-action 'none'",
  "frame-ancestors 'none'",
].join("; ");

const files = walk(dir)
  .map((f) => relative(dir, f))
  .filter((f) => !f.startsWith(".well-known/") && f !== "_headers")
  .sort();
const manifest = files
  .map(
    (f) =>
      `${createHash("sha256")
        .update(readFileSync(join(dir, f)))
        .digest("hex")}  ${f}\n`,
  )
  .join("");
const bundle = createHash("sha256").update(manifest).digest("hex");

mkdirSync(join(dir, ".well-known"), { recursive: true });
writeFileSync(join(dir, ".well-known/bundle.txt"), manifest);
writeFileSync(
  join(dir, ".well-known/provenance.json"),
  JSON.stringify({ commit, bundle }, null, 2) + "\n",
);
writeFileSync(
  join(dir, "_headers"),
  `/*\n  X-Uploadnt-Commit: ${commit}\n  X-Uploadnt-Bundle: ${bundle}\n  Content-Security-Policy: ${csp}\n  Referrer-Policy: no-referrer\n  X-Content-Type-Options: nosniff\n`,
);
console.log(`bundle ${bundle} commit ${commit}`);
