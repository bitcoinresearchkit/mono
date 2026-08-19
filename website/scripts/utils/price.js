let _latest = /** @type {number | null} */ (null);

/** @type {Set<(price: number) => void>} */
const listeners = new Set();

/** @param {(price: number) => void} callback */
export function onPrice(callback) {
  listeners.add(callback);
  if (_latest !== null) callback(_latest);
  return () => listeners.delete(callback);
}

export function latestPrice() {
  return _latest;
}

/** @param {BitviewClient} brk */
export function initPrice(brk) {
  async function poll() {
    try {
      const price = await brk.getLivePrice();
      if (price !== _latest) {
        _latest = price;
        listeners.forEach((cb) => cb(price));
      }
    } catch (e) {
      console.error("price poll:", e);
    }
  }

  poll();
  setInterval(poll, 1_000);
  document.addEventListener("visibilitychange", () => {
    !document.hidden && poll();
  });
}
