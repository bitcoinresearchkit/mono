#!/usr/bin/env node

import { readFile } from "node:fs/promises";

const DEFAULT_BASE = "http://127.0.0.1:3110";
const DEFAULT_MANIFEST = "crates/bitview_mcp/generated/manifest.json";
const SAMPLE_LIMIT = 256;

function option(name, fallback) {
  const index = process.argv.indexOf(name);
  return index === -1 ? fallback : process.argv[index + 1];
}

const base = option("--base", DEFAULT_BASE).replace(/\/$/, "");
const mcpBase = option("--mcp", "")?.replace(/\/$/, "");
const manifestPath = option("--manifest", DEFAULT_MANIFEST);
const timeoutMs = Number(option("--timeout-ms", "60000"));
const manifest = JSON.parse(await readFile(manifestPath, "utf8"));

function jsonEqual(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function resolveRef(schema, root) {
  if (!schema?.$ref?.startsWith("#/")) return schema;
  return schema.$ref
    .slice(2)
    .split("/")
    .map((part) => part.replaceAll("~1", "/").replaceAll("~0", "~"))
    .reduce((value, part) => value?.[part], root);
}

function valueType(value) {
  if (value === null) return "null";
  if (Array.isArray(value)) return "array";
  if (Number.isInteger(value)) return "integer";
  return typeof value === "number" ? "number" : typeof value;
}

function typeMatches(value, type) {
  if (Array.isArray(type)) return type.some((entry) => typeMatches(value, entry));
  if (type === "null") return value === null;
  if (type === "array") return Array.isArray(value);
  if (type === "object") return value !== null && typeof value === "object" && !Array.isArray(value);
  if (type === "integer") return Number.isInteger(value);
  if (type === "number") return typeof value === "number" && Number.isFinite(value);
  return typeof value === type;
}

function sampledEntries(values) {
  if (values.length <= SAMPLE_LIMIT) return values.map((value, index) => [index, value]);
  const entries = [];
  for (let sample = 0; sample < SAMPLE_LIMIT; sample += 1) {
    const index = Math.round((sample * (values.length - 1)) / (SAMPLE_LIMIT - 1));
    entries.push([index, values[index]]);
  }
  return entries;
}

function validate(value, schema, root = schema, path = "$", seen = new Set()) {
  if (schema === true || schema === undefined) return [];
  if (schema === false) return [`${path}: schema rejects every value`];
  if (!schema || typeof schema !== "object") return [];

  if (schema.$ref) {
    const resolved = resolveRef(schema, root);
    if (!resolved) return [`${path}: unresolved schema reference ${schema.$ref}`];
    const key = `${schema.$ref}:${path}`;
    if (seen.has(key)) return [];
    const nextSeen = new Set(seen).add(key);
    const siblings = { ...schema };
    delete siblings.$ref;
    return [
      ...validate(value, resolved, root, path, nextSeen),
      ...validate(value, siblings, root, path, nextSeen),
    ];
  }

  const errors = [];
  if (schema.const !== undefined && !jsonEqual(value, schema.const)) {
    errors.push(`${path}: expected constant ${JSON.stringify(schema.const)}`);
  }
  if (schema.enum && !schema.enum.some((candidate) => jsonEqual(value, candidate))) {
    errors.push(`${path}: ${JSON.stringify(value)} is outside the enum`);
  }
  if (schema.type && !typeMatches(value, schema.type)) {
    errors.push(`${path}: expected ${JSON.stringify(schema.type)}, got ${valueType(value)}`);
    return errors;
  }

  for (const child of schema.allOf ?? []) {
    errors.push(...validate(value, child, root, path, seen));
  }
  if (schema.anyOf) {
    const variants = schema.anyOf.map((child) => validate(value, child, root, path, seen));
    if (!variants.some((variant) => variant.length === 0)) {
      errors.push(`${path}: no anyOf branch matched (${variants.map((v) => v[0]).join("; ")})`);
      return errors;
    }
  }
  if (schema.oneOf) {
    const matches = schema.oneOf.filter((child) => validate(value, child, root, path, seen).length === 0);
    if (matches.length !== 1) errors.push(`${path}: expected exactly one oneOf match, got ${matches.length}`);
  }
  if (schema.not && validate(value, schema.not, root, path, seen).length === 0) {
    errors.push(`${path}: value matched a forbidden schema`);
  }

  if (typeof value === "string") {
    if (schema.minLength !== undefined && value.length < schema.minLength) {
      errors.push(`${path}: string is shorter than ${schema.minLength}`);
    }
    if (schema.maxLength !== undefined && value.length > schema.maxLength) {
      errors.push(`${path}: string is longer than ${schema.maxLength}`);
    }
    if (schema.pattern && !new RegExp(schema.pattern).test(value)) {
      errors.push(`${path}: string does not match ${schema.pattern}`);
    }
  }

  if (typeof value === "number") {
    if (schema.minimum !== undefined && value < schema.minimum) errors.push(`${path}: below minimum`);
    if (schema.maximum !== undefined && value > schema.maximum) errors.push(`${path}: above maximum`);
    if (schema.exclusiveMinimum !== undefined && value <= schema.exclusiveMinimum) {
      errors.push(`${path}: below exclusive minimum`);
    }
    if (schema.exclusiveMaximum !== undefined && value >= schema.exclusiveMaximum) {
      errors.push(`${path}: above exclusive maximum`);
    }
  }

  if (Array.isArray(value)) {
    if (schema.minItems !== undefined && value.length < schema.minItems) errors.push(`${path}: too few items`);
    if (schema.maxItems !== undefined && value.length > schema.maxItems) errors.push(`${path}: too many items`);
    for (let index = 0; index < (schema.prefixItems?.length ?? 0) && index < value.length; index += 1) {
      errors.push(...validate(value[index], schema.prefixItems[index], root, `${path}[${index}]`, seen));
    }
    if (schema.items && typeof schema.items === "object") {
      for (const [index, item] of sampledEntries(value)) {
        errors.push(...validate(item, schema.items, root, `${path}[${index}]`, seen));
      }
    } else if (schema.items === false && value.length > (schema.prefixItems?.length ?? 0)) {
      errors.push(`${path}: contains items beyond prefixItems`);
    }
  }

  if (value !== null && typeof value === "object" && !Array.isArray(value)) {
    for (const property of schema.required ?? []) {
      if (!Object.hasOwn(value, property)) errors.push(`${path}: missing required property ${property}`);
    }
    for (const [property, child] of Object.entries(schema.properties ?? {})) {
      if (Object.hasOwn(value, property)) {
        errors.push(...validate(value[property], child, root, `${path}.${property}`, seen));
      }
    }
    if (schema.additionalProperties === false) {
      const documented = new Set(Object.keys(schema.properties ?? {}));
      for (const property of Object.keys(value)) {
        if (!documented.has(property)) errors.push(`${path}: unexpected property ${property}`);
      }
    } else if (schema.additionalProperties && typeof schema.additionalProperties === "object") {
      const documented = new Set(Object.keys(schema.properties ?? {}));
      for (const [property, child] of Object.entries(value)) {
        if (!documented.has(property)) {
          errors.push(...validate(child, schema.additionalProperties, root, `${path}.${property}`, seen));
        }
      }
    }
  }

  return errors;
}

function coverageGaps(value, schema, root = schema, path = "$", seen = new Set()) {
  if (!schema || typeof schema !== "object" || schema === true) return [];
  if (schema.$ref) {
    const resolved = resolveRef(schema, root);
    if (!resolved) return [];
    const key = `${schema.$ref}:${path}`;
    if (seen.has(key)) return [];
    return coverageGaps(value, resolved, root, path, new Set(seen).add(key));
  }
  if (schema.anyOf || schema.oneOf) {
    const branches = schema.anyOf ?? schema.oneOf;
    const branch = branches.find((child) => validate(value, child, root, path).length === 0);
    return branch ? coverageGaps(value, branch, root, path, seen) : [];
  }
  const gaps = [];
  for (const child of schema.allOf ?? []) gaps.push(...coverageGaps(value, child, root, path, seen));
  if (Array.isArray(value)) {
    if (schema.items && typeof schema.items === "object") {
      for (const [index, item] of sampledEntries(value)) {
        gaps.push(...coverageGaps(item, schema.items, root, `${path}[${index}]`, seen));
      }
    }
    return gaps;
  }
  if (value === null || typeof value !== "object") return gaps;

  const properties = schema.properties ?? {};
  if (Object.keys(properties).length > 0 && schema.additionalProperties === undefined) {
    for (const property of Object.keys(value)) {
      if (!Object.hasOwn(properties, property)) gaps.push(`${path}.${property}: emitted but undocumented`);
    }
  }
  for (const [property, child] of Object.entries(properties)) {
    if (Object.hasOwn(value, property)) {
      gaps.push(...coverageGaps(value[property], child, root, `${path}.${property}`, seen));
    }
  }
  return gaps;
}

async function request(path, { allowError = false } = {}) {
  let lastError;
  for (let attempt = 1; attempt <= 4; attempt += 1) {
    try {
      const response = await fetch(`${base}${path}`, { signal: AbortSignal.timeout(timeoutMs) });
      const body = new Uint8Array(await response.arrayBuffer());
      if (!response.ok && !allowError) {
        const detail = new TextDecoder().decode(body).slice(0, 500);
        throw new Error(`HTTP ${response.status}: ${detail}`);
      }
      return { response, body };
    } catch (error) {
      lastError = error;
      if (attempt < 4) await new Promise((resolve) => setTimeout(resolve, 250 * attempt));
    }
  }
  throw lastError;
}

async function mcpCall(operation, args, id) {
  let encodedArgs = JSON.stringify(args);
  if (operation.tool.name === "get_block_template_diff") {
    encodedArgs = encodedArgs.replace(/"hash":\d+/, `"hash":${mempoolHash}`);
  }
  const body = `{"jsonrpc":"2.0","id":${id},"method":"tools/call","params":{"name":${JSON.stringify(operation.tool.name)},"arguments":${encodedArgs},"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"schema-audit","version":"1.0"},"io.modelcontextprotocol/clientCapabilities":{}}}}`;
  let lastError;
  for (let attempt = 1; attempt <= 4; attempt += 1) {
    try {
      const response = await fetch(`${mcpBase}/`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Accept: "application/json, text/event-stream",
          "MCP-Protocol-Version": "2026-07-28",
          "Mcp-Method": "tools/call",
          "Mcp-Name": operation.tool.name,
        },
        body,
        signal: AbortSignal.timeout(timeoutMs),
      });
      const payload = await response.json();
      if (!response.ok) throw new Error(`HTTP ${response.status}: ${JSON.stringify(payload).slice(0, 500)}`);
      if (payload.error) throw new Error(`JSON-RPC ${payload.error.code}: ${payload.error.message}`);
      if (payload.result?.isError) {
        const detail = payload.result.content?.map((item) => item.text).filter(Boolean).join(" ");
        throw new Error(detail || "tool returned isError=true");
      }
      return payload.result;
    } catch (error) {
      lastError = error;
      if (attempt < 4) await new Promise((resolve) => setTimeout(resolve, 250 * attempt));
    }
  }
  throw lastError;
}

async function json(path) {
  const { body } = await request(path);
  return JSON.parse(new TextDecoder().decode(body));
}

async function text(path) {
  const { body } = await request(path);
  return new TextDecoder().decode(body);
}

const blocks = await json("/api/blocks");
const block = blocks[0];
const blockTxids = await json(`/api/block/${block.id}/txids`);
const confirmedTxid = blockTxids[Math.min(1, blockTxids.length - 1)];
const confirmedTx = await json(`/api/tx/${confirmedTxid}`);
const address =
  confirmedTx.vout.find((output) => output.scriptpubkey_address)?.scriptpubkey_address ??
  "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
const addressTxs = await json(`/api/address/${encodeURIComponent(address)}/txs/chain`);
const addressAfterTxid = addressTxs.at(-1)?.txid ?? confirmedTxid;
const mempoolTxids = await json("/api/mempool/txids");
const mempoolTxid = mempoolTxids[0] ?? confirmedTxid;
const mempoolHash = (await text("/api/mempool/hash")).trim().replace(/^"|"$/g, "");
const blockV1 = await json(`/api/v1/block/${block.id}`);
const poolSlug = blockV1.extras?.pool?.slug ?? "unknown";
const urpdDates = await json("/api/urpd/all/dates");
const urpdDate = urpdDates.at(-1);

const common = {
  address,
  after_txid: addressAfterTxid,
  addr_type: "p2pkh",
  prefix: "62e907b15c",
  hash: block.id,
  height: block.height,
  timestamp: block.timestamp,
  time_period: "24h",
  index: 0,
  start_index: 0,
  txid: confirmedTxid,
  slug: poolSlug,
  block_count: 10,
  series: "price_close",
  cohort: "all",
  date: urpdDate,
  point: String(block.height),
  vout: 0,
  q: "price_close",
  "txId[]": [mempoolTxid],
};

const overrides = {
  // JSON Schema integers and serde_json can carry a u64 exactly, but JavaScript
  // cannot. Validate with a number while preserving the exact path digits below.
  get_block_template_diff: { hash: Number(mempoolHash) },
  get_cpfp: { txid: mempoolTxid },
  get_series: { series: "price_close", index: "day1", limit: 2 },
  get_series_bulk: { series: "price_close,market_cap", index: "day1", limit: 2 },
  get_series_data: { series: "price_close", index: "day1", limit: 2 },
  get_series_info: { series: "price_close" },
  get_series_latest: { series: "price_close", index: "day1" },
  get_series_len: { series: "price_close", index: "day1" },
  get_series_version: { series: "price_close", index: "day1" },
  get_tx_by_index: { index: confirmedTx.index },
  get_tx_rbf: { txid: mempoolTxid },
};

function argumentsFor(operation) {
  const required = operation.tool.inputSchema?.required ?? [];
  const optional = overrides[operation.tool.name] ?? {};
  const args = { ...optional };
  for (const name of required) {
    if (!Object.hasOwn(args, name)) args[name] = common[name];
  }
  return args;
}

function pathFor(operation, args) {
  let path = operation.http.path;
  const query = new URLSearchParams();
  for (const parameter of operation.http.parameters) {
    const value =
      operation.tool.name === "get_block_template_diff" && parameter.name === "hash"
        ? mempoolHash
        : args[parameter.name];
    if (value === undefined) continue;
    if (parameter.location === "path") {
      path = path.replace(`{${parameter.name}}`, encodeURIComponent(String(value)));
    } else {
      for (const item of Array.isArray(value) ? value : [value]) query.append(parameter.name, String(item));
    }
  }
  const suffix = query.toString();
  return suffix ? `${path}?${suffix}` : path;
}

function payloadFrom(response, body) {
  const contentType = response.headers.get("content-type")?.split(";", 1)[0] ?? "";
  const decoded = new TextDecoder().decode(body);
  if (contentType === "application/json" || contentType.endsWith("+json")) return JSON.parse(decoded);
  if (contentType.startsWith("text/")) return decoded;
  return body;
}

const passed = [];
const schemaFailures = [];
const documentationGaps = [];
const requestFailures = [];
const untyped = [];
const mcpFailures = [];

for (const operation of manifest.operations) {
  const name = operation.tool.name;
  const args = argumentsFor(operation);
  const inputErrors = validate(args, operation.tool.inputSchema);
  if (inputErrors.length > 0) {
    schemaFailures.push({ name, kind: "input fixture", errors: inputErrors });
    continue;
  }
  try {
    const path = pathFor(operation, args);
    const { response, body } = await request(path);
    if (!operation.tool.outputSchema) {
      untyped.push(name);
      passed.push(name);
      continue;
    }
    const payload = payloadFrom(response, body);
    const errors = validate(payload, operation.tool.outputSchema);
    const gaps = coverageGaps(payload, operation.tool.outputSchema);
    if (errors.length > 0) schemaFailures.push({ name, kind: "output", errors: [...new Set(errors)] });
    if (gaps.length > 0) documentationGaps.push({ name, gaps: [...new Set(gaps)] });
    if (errors.length === 0 && gaps.length === 0) passed.push(name);
  } catch (error) {
    requestFailures.push({ name, error: String(error) });
  }
}

if (mcpBase) {
  let id = 1000;
  for (const operation of manifest.operations) {
    const name = operation.tool.name;
    try {
      const result = await mcpCall(operation, argumentsFor(operation), id++);
      if (operation.tool.outputSchema) {
        if (result.structuredContent === undefined) {
          mcpFailures.push({ name, errors: ["$: missing structuredContent"] });
          continue;
        }
        const errors = validate(result.structuredContent, operation.tool.outputSchema);
        const gaps = coverageGaps(result.structuredContent, operation.tool.outputSchema);
        if (errors.length > 0 || gaps.length > 0) {
          mcpFailures.push({ name, errors: [...new Set([...errors, ...gaps])] });
        }
      }
    } catch (error) {
      mcpFailures.push({ name, errors: [String(error)] });
    }
  }
}

console.log(`Catalog operations: ${manifest.operations.length}`);
console.log(`Fully conforming: ${passed.length}`);
console.log(`Schema failures: ${schemaFailures.length}`);
console.log(`Undocumented output fields: ${documentationGaps.length}`);
console.log(`Request failures: ${requestFailures.length}`);
console.log(`Operations without output schemas: ${untyped.length}`);
if (mcpBase) console.log(`Live MCP failures: ${mcpFailures.length}`);

for (const failure of schemaFailures) {
  console.log(`\nSCHEMA ${failure.name} (${failure.kind})`);
  for (const error of failure.errors.slice(0, 20)) console.log(`  ${error}`);
}
for (const failure of documentationGaps) {
  console.log(`\nCOVERAGE ${failure.name}`);
  for (const gap of failure.gaps.slice(0, 20)) console.log(`  ${gap}`);
}
for (const failure of requestFailures) console.log(`\nREQUEST ${failure.name}\n  ${failure.error}`);
for (const failure of mcpFailures) {
  console.log(`\nMCP ${failure.name}`);
  for (const error of failure.errors.slice(0, 20)) console.log(`  ${error}`);
}
if (untyped.length > 0) console.log(`\nUNTYPED\n  ${untyped.join(", ")}`);

if (
  schemaFailures.length > 0 ||
  documentationGaps.length > 0 ||
  requestFailures.length > 0 ||
  mcpFailures.length > 0
) {
  process.exitCode = 1;
}
