import { bitview } from "../../../utils/client.js";

const MAX_TEXT = 2_000;
const MAX_ARRAY = 8;
const ARRAY_SAMPLE = 4;
const MAX_KEYS = 64;
const MAX_DEPTH = 6;

/** @param {unknown} value */
function hasValue(value) {
  return value !== undefined && value !== null && value !== "";
}

/** @param {unknown} value @param {number} depth @param {{ truncated: boolean }} state @returns {unknown} */
function compact(value, depth, state) {
  if (depth >= MAX_DEPTH) {
    state.truncated = true;
    return "[nested value omitted]";
  }
  if (typeof value === "string" && value.length > MAX_TEXT) {
    state.truncated = true;
    return `${value.slice(0, MAX_TEXT)}…`;
  }
  if (Array.isArray(value)) {
    if (value.length <= MAX_ARRAY) {
      return value.map((item) => compact(item, depth + 1, state));
    }
    state.truncated = true;
    return {
      count: value.length,
      sample: value.slice(0, ARRAY_SAMPLE).map((item) => compact(item, depth + 1, state)),
    };
  }
  if (value && typeof value === "object") {
    const entries = Object.entries(value);
    if (entries.length > MAX_KEYS) state.truncated = true;
    return Object.fromEntries(
      entries.slice(0, MAX_KEYS).map(([key, item]) => [
        key,
        compact(item, depth + 1, state),
      ]),
    );
  }
  return value;
}

/** @param {unknown} value @param {import("./index.js").ApiParameter} parameter */
function parameterValue(value, parameter) {
  const type = parameter.primitive ?? parameter.valueType ?? parameter.type;
  if (type.includes("integer")) {
    const number = Number(value);
    if (!Number.isInteger(number)) throw new Error(`${parameter.name} must be an integer`);
    return String(number);
  }
  if (type.includes("number")) {
    const number = Number(value);
    if (!Number.isFinite(number)) throw new Error(`${parameter.name} must be a number`);
    return String(number);
  }
  if (type.includes("boolean")) {
    if (value !== true && value !== false && value !== "true" && value !== "false") {
      throw new Error(`${parameter.name} must be true or false`);
    }
    return String(value);
  }
  const string = Array.isArray(value) ? value.join(",") : String(value);
  if (parameter.enum?.length && !parameter.enum.map(String).includes(string)) {
    throw new Error(`${parameter.name} must be one of: ${parameter.enum.join(", ")}`);
  }
  return string;
}

/**
 * @param {import("./index.js").ApiOperation} operation
 * @param {Record<string, unknown>} arguments_
 * @param {AbortSignal} signal
 */
export async function executeApi(operation, arguments_, signal) {
  if (operation.method !== "GET" || !operation.path.startsWith("/")) {
    throw new Error("Only generated read-only API operations are allowed");
  }
  const allowed = new Set(operation.parameters.map((parameter) => parameter.name));
  for (const key of Object.keys(arguments_)) {
    if (!allowed.has(key)) throw new Error(`Unexpected API parameter: ${key}`);
  }

  let path = operation.path;
  const query = new URLSearchParams();
  for (const parameter of operation.parameters) {
    const value = arguments_[parameter.name];
    if (!hasValue(value)) {
      if (parameter.required) throw new Error(`${parameter.name} is required`);
      continue;
    }
    const encoded = parameterValue(value, parameter);
    if (parameter.in === "path") {
      path = path.replace(`{${parameter.name}}`, encodeURIComponent(encoded));
    } else if (parameter.in === "query") {
      query.set(parameter.name, encoded);
    }
  }
  if (/\{[^}]+\}/.test(path)) throw new Error("A required path parameter is missing");
  if (query.size) path += `?${query}`;

  const response = await bitview.get(path, { signal });
  const contentType = response.headers.get("content-type") ?? operation.response.contentType;
  const raw = contentType.includes("json") ? await response.json() : await response.text();
  const state = { truncated: false };
  const data = compact(raw, 0, state);
  return {
    operation: {
      key: operation.key,
      method: operation.method,
      path: operation.path,
      summary: operation.summary || operation.label,
      description: operation.description,
      parameters: operation.parameters,
      response: operation.response,
    },
    arguments: Object.fromEntries(
      Object.entries(arguments_).filter(([, value]) => hasValue(value)),
    ),
    requestPath: path,
    data,
    truncated: state.truncated,
  };
}
