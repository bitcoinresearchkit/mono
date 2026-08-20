import { readParam, writeParam } from "./url.js";
import { readStored, writeToStorage } from "./storage.js";
import { debounce } from "./timing.js";

/**
 * @template T
 * @param {Object} args
 * @param {T} args.defaultValue
 * @param {string} [args.storageKey]
 * @param {string} [args.urlKey]
 * @param {(v: T) => string} args.serialize
 * @param {(s: string) => T} args.deserialize
 * @param {boolean} [args.saveDefaultValue]
 * @param {(v: T) => void} [args.onChange]
 */
export function createPersistedValue({
  defaultValue,
  storageKey,
  urlKey,
  serialize,
  deserialize,
  saveDefaultValue = false,
  onChange,
}) {
  const defaultSerialized = serialize(defaultValue);

  // Read: URL > localStorage > default
  let serialized = urlKey ? readParam(urlKey) : null;
  if (serialized === null && storageKey) {
    serialized = readStored(storageKey);
  }
  let value = serialized !== null ? deserialize(serialized) : defaultValue;

  /** @param {T} v */
  const write = (v) => {
    const s = serialize(v);
    const isDefault = s === defaultSerialized;

    if (storageKey) {
      if (!isDefault || saveDefaultValue) {
        writeToStorage(storageKey, s);
      } else {
        writeToStorage(storageKey, null);
      }
    }

    if (urlKey) {
      writeParam(urlKey, !isDefault || saveDefaultValue ? s : null);
    }
  };

  const debouncedWrite = debounce(write, 250);

  // Write initial value
  write(value);

  return {
    get value() {
      return value;
    },
    /** @param {T} v */
    set(v) {
      value = v;
      debouncedWrite(v);
      onChange?.(v);
    },
    /** @param {T} v */
    setImmediate(v) {
      value = v;
      write(v);
      onChange?.(v);
    },
  };
}

/**
 * @template T
 * @typedef {ReturnType<typeof createPersistedValue<T>>} PersistedValue
 */
