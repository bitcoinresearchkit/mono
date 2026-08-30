const MISSING = Symbol("missing cohort");

/**
 * @typedef {null | undefined | string | number | boolean | bigint | symbol} ProjectablePrimitive
 * @typedef {(...args: never[]) => void} ProjectableFunction
 * @typedef {{ [key: string]: ProjectableValue }} ProjectableRecord
 * @typedef {ProjectablePrimitive | ProjectableFunction | ProjectableRecord} ProjectableValue
 */

/**
 * @param {ProjectableValue} value
 * @returns {value is ProjectableRecord}
 */
function isObject(value) {
  return value !== null && typeof value === "object";
}

/**
 * Series patterns contain lazy endpoint getters, so they are leaves while the
 * metric-first cohort tree is being projected.
 *
 * @param {ProjectableRecord} value
 * @returns {boolean}
 */
function isSeriesPattern(value) {
  return (
    typeof value.name === "string" &&
    typeof value.indexes === "function" &&
    typeof value.get === "function"
  );
}

/**
 * @param {{ value: ProjectableValue, path: readonly string[] }} args
 * @returns {ProjectableValue | typeof MISSING}
 */
function readPath({ value, path }) {
  let current = value;
  for (const key of path) {
    if (!isObject(current) || !(key in current)) return MISSING;
    current = current[key];
  }
  return current;
}

/**
 * Aggregate-only metrics use `sth`/`lth` directly instead of a nested `term`
 * branch.
 *
 * @param {{ value: ProjectableRecord, path: readonly string[] }} args
 * @returns {string | undefined}
 */
function directKey({ value, path }) {
  if (path[0] in value) return path[0];
  if (path.length !== 2 || path[0] !== "term") return undefined;
  const aliases =
    path[1] === "short"
      ? ["short", "sth"]
      : path[1] === "long"
        ? ["long", "lth"]
        : [];
  return aliases.find((alias) => alias in value);
}

/**
 * @param {{ value: ProjectableValue, path: readonly string[] }} args
 * @returns {ProjectableValue | typeof MISSING}
 */
function project({ value, path }) {
  if (!isObject(value) || isSeriesPattern(value)) return MISSING;

  const selectedKey = directKey({ value, path });
  const selected = selectedKey
    ? selectedKey === path[0]
      ? readPath({ value: value[selectedKey], path: path.slice(1) })
      : value[selectedKey]
    : MISSING;
  /** @type {ProjectableRecord} */
  const result = {};
  for (const [key, child] of Object.entries(value)) {
    if (key === selectedKey) continue;
    const projected = project({ value: child, path });
    if (projected !== MISSING) result[key] = projected;
  }

  if (selected === MISSING) {
    return Object.keys(result).length ? result : MISSING;
  }
  if (!Object.keys(result).length || !isObject(selected)) return selected;
  return { ...selected, ...result };
}

/**
 * Convert the generated metric-first distribution tree into the cohort-first
 * view consumed by the chart option builders.
 *
 * @template {object} Tree
 * @template {string} Path
 * @param {{ tree: Tree, path: Path }} args
 * @returns {import("./cohort-tree-types.js").ProjectCohortPath<Tree, Path>}
 */
export function selectCohortTree({ tree, path }) {
  const root = /** @type {ProjectableRecord} */ (tree);
  const segments = path.split(".");
  /** @type {ProjectableRecord | undefined} */
  let materialized;

  /**
   * Resolve one output path without projecting unrelated siblings.
   *
   * @param {ProjectableValue} value
   * @param {readonly string[]} outputPath
   * @returns {ProjectableValue | typeof MISSING}
   */
  function resolve(value, outputPath) {
    if (!outputPath.length) {
      if (!isObject(value) || isSeriesPattern(value)) return value;
      const selectedKey = directKey({ value, path: segments });
      if (!selectedKey) return value;
      return selectedKey === segments[0]
        ? readPath({ value: value[selectedKey], path: segments.slice(1) })
        : value[selectedKey];
    }
    if (!isObject(value) || isSeriesPattern(value)) return MISSING;

    const selectedKey = directKey({ value, path: segments });
    const selected = selectedKey
      ? selectedKey === segments[0]
        ? readPath({ value: value[selectedKey], path: segments.slice(1) })
        : value[selectedKey]
      : MISSING;
    const [key, ...tail] = outputPath;
    const projected =
      key !== selectedKey && key in value ? resolve(value[key], tail) : MISSING;

    if (projected !== MISSING) return projected;
    return selected !== MISSING
      ? readPath({ value: selected, path: outputPath })
      : MISSING;
  }

  function materialize() {
    if (materialized) return materialized;
    const selected = project({ value: root, path: segments });
    if (selected === MISSING || !isObject(selected)) {
      throw new Error(`Cohort path not found: ${path}`);
    }
    materialized = selected;
    return selected;
  }

  /**
   * @param {readonly string[]} outputPath
   * @returns {ProjectableRecord}
   */
  function view(outputPath) {
    /** @type {Map<PropertyKey, ProjectableValue | typeof MISSING>} */
    const cache = new Map();

    /** @param {PropertyKey} key */
    function child(key) {
      if (cache.has(key)) return cache.get(key);
      if (typeof key !== "string") return MISSING;

      const childPath = [...outputPath, key];
      const resolved = materialized
        ? readPath({ value: materialized, path: childPath })
        : resolve(root, childPath);
      const value =
        resolved !== MISSING && isObject(resolved) ? view(childPath) : resolved;
      cache.set(key, value);
      return value;
    }

    return new Proxy(
      {},
      {
        get(_target, key) {
          const value = child(key);
          return value === MISSING ? undefined : value;
        },
        has(_target, key) {
          return child(key) !== MISSING;
        },
        ownKeys() {
          const value = readPath({ value: materialize(), path: outputPath });
          return isObject(value) ? Reflect.ownKeys(value) : [];
        },
        getOwnPropertyDescriptor(_target, key) {
          return child(key) === MISSING
            ? undefined
            : { configurable: true, enumerable: true };
        },
      },
    );
  }

  return /** @type {import("./cohort-tree-types.js").ProjectCohortPath<Tree, Path>} */ (
    view([])
  );
}
