/**
 * Compute a value once, when it is first needed.
 *
 * @template T
 * @param {() => T} create
 * @returns {() => T}
 */
export function lazy(create) {
  /** @type {T | undefined} */
  let value;
  let ready = false;

  return () => {
    if (!ready) {
      value = create();
      ready = true;
    }
    return /** @type {T} */ (value);
  };
}

/**
 * Keep a group's name available for routing while deferring its chart tree.
 *
 * @param {string} name
 * @param {() => PartialOptionsGroup} create
 * @returns {PartialOptionsGroup}
 */
export function lazyGroup(name, create) {
  const get = lazy(create);

  return {
    name,
    get tree() {
      return get().tree;
    },
  };
}
