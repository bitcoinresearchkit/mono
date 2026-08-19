import { mkdir, rename, unlink, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";

const repository = "bitcoinresearchkit/brk";
const revision = process.argv[2] ?? "main";
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const output = resolve(
  process.argv[3] ?? `${root}/website_next/ask/tools/source/catalog.jsonl.gz`,
);
const temporaryOutput = `${output}.${process.pid}.tmp`;
const maxFileSize = 128_000;
const concurrency = 12;
const extensions = new Set([
  "css",
  "html",
  "js",
  "json",
  "md",
  "mjs",
  "py",
  "rs",
  "sh",
  "toml",
  "ts",
  "yaml",
  "yml",
]);
const excludedPrefixes = [
  ".git/",
  ".github/",
  "docs/ai/",
  "modules/",
  "packages/bitview_client/bitview_client/",
  "target/",
  "website/assets/",
  "website_next/modules/",
];
const excludedFiles = new Set([
  "crates/bitview_server/src/api/scalar.js",
  "docs/CHANGELOG.md",
  "website/scripts/options/scalar.js",
]);
const headers = {
  Accept: "application/vnd.github+json",
  "User-Agent": "brk-ask-source-builder",
};

async function fetchResponse(url, options = {}) {
  let lastError;
  for (let attempt = 0; attempt < 3; attempt += 1) {
    try {
      const response = await fetch(url, options);
      if (response.ok) return response;
      lastError = new Error(`${response.status} ${response.statusText}: ${url}`);
      if (response.status < 500 && response.status !== 429) break;
    } catch (error) {
      lastError = error;
    }
    await new Promise((resolveDelay) =>
      setTimeout(resolveDelay, 250 * 2 ** attempt),
    );
  }
  throw lastError;
}

async function fetchJson(url) {
  return fetchResponse(url, { headers }).then((response) => response.json());
}

function isUsefulSource(entry) {
  if (entry.type !== "blob" || !entry.size || entry.size > maxFileSize) {
    return false;
  }
  const extension = entry.path.slice(entry.path.lastIndexOf(".") + 1).toLowerCase();
  return (
    extensions.has(extension) &&
    !excludedFiles.has(entry.path) &&
    !excludedPrefixes.some((prefix) => entry.path.startsWith(prefix))
  );
}

function rawUrl(commit, filePath) {
  const encodedPath = filePath.split("/").map(encodeURIComponent).join("/");
  return `https://raw.githubusercontent.com/${repository}/${commit}/${encodedPath}`;
}

async function mapConcurrent(items, mapper) {
  const results = new Array(items.length);
  let nextIndex = 0;
  async function worker() {
    while (nextIndex < items.length) {
      const index = nextIndex;
      nextIndex += 1;
      results[index] = await mapper(items[index], index);
    }
  }
  await Promise.all(
    Array.from({ length: Math.min(concurrency, items.length) }, worker),
  );
  return results;
}

async function main() {
  const commitInfo = await fetchJson(
    `https://api.github.com/repos/${repository}/commits/${encodeURIComponent(revision)}`,
  );
  const commit = commitInfo.sha;
  if (!commit) throw new Error(`Could not resolve revision: ${revision}`);

  const treeInfo = await fetchJson(
    `https://api.github.com/repos/${repository}/git/trees/${commit}?recursive=1`,
  );
  if (treeInfo.truncated) {
    throw new Error("GitHub returned a truncated repository tree");
  }

  const entries = treeInfo.tree.filter(isUsefulSource);
  entries.sort((left, right) => left.path.localeCompare(right.path));
  const files = await mapConcurrent(entries, async (entry, index) => {
    const response = await fetchResponse(rawUrl(commit, entry.path));
    const source = await response.text();
    if ((index + 1) % 250 === 0 || index + 1 === entries.length) {
      console.error(`Fetched ${index + 1}/${entries.length}`);
    }
    return JSON.stringify([entry.path, source]);
  });

  const header = JSON.stringify({
    repository,
    revision: commit,
    count: files.length,
  });
  const raw = Buffer.from([header, ...files].join("\n"));
  const compressed = gzipSync(raw, { level: 9 });

  await mkdir(dirname(output), { recursive: true });
  try {
    await writeFile(temporaryOutput, compressed);
    await rename(temporaryOutput, output);
  } catch (error) {
    await unlink(temporaryOutput).catch(() => {});
    throw error;
  }

  console.log(
    JSON.stringify(
      {
        revision: commit,
        files: files.length,
        rawBytes: raw.length,
        compressedBytes: compressed.length,
        output,
      },
      null,
      2,
    ),
  );
}

await main();
