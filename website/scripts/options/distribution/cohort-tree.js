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
  const selected = project({ value: root, path: path.split(".") });
  if (selected === MISSING || !isObject(selected)) {
    throw new Error(`Cohort path not found: ${path}`);
  }
  return /** @type {import("./cohort-tree-types.js").ProjectCohortPath<Tree, Path>} */ (
    selected
  );
}
