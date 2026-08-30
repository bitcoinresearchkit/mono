/**
 * Run chart-title matching off the main thread so loading and indexing the
 * complete search catalog cannot interrupt typing or rendering.
 *
 * @returns {(needle: string) => Promise<Array<[title: string, href: string, blank: boolean]>>}
 */
export function createChartSearch() {
  const worker = new Worker(new URL("./search-worker.js", import.meta.url), {
    type: "module",
  });
  let nextId = 0;
  /** @type {Map<number, { resolve: (results: Array<[string, string, boolean]>) => void, reject: (error: Error) => void }>} */
  const pending = new Map();

  worker.addEventListener("message", (event) => {
    const { id, results, error } =
      /** @type {{ id: number, results?: Array<[string, string, boolean]>, error?: string }} */ (
        event.data
      );
    const request = pending.get(id);
    if (!request) return;
    pending.delete(id);
    if (error) {
      request.reject(new Error(error));
    } else {
      request.resolve(results || []);
    }
  });

  return (needle) =>
    new Promise((resolve, reject) => {
      const id = nextId++;
      pending.set(id, { resolve, reject });
      worker.postMessage({ id, needle });
    });
}
