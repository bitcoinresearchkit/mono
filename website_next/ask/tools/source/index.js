import { WorkerClient } from "../worker-client.js";

const WORKER_URL = import.meta.resolve("./worker.js");

export class AskSource {
  #client = new WorkerClient(WORKER_URL, {
    failed: "The source worker failed",
    stopped: "Source search stopped",
  });

  prewarm() {
    return this.#client.request("prewarm", {});
  }

  /**
   * @param {string} query
   * @param {string | undefined} path
   * @param {"definition" | "implementation" | "usage" | "availability" | undefined} focus
   * @param {((progress: { loaded: number, total: number }) => void) | undefined} [onProgress]
   * @returns {Promise<{ revision: string, matches: ReturnType<typeof import("./search.js").searchSource> }>}
   */
  search(query, path, focus, onProgress) {
    return this.#client.request(
      "search",
      { query, path, focus },
      onProgress
        ? ({ loaded, total }) => onProgress({ loaded, total })
        : undefined,
    );
  }

  terminate() {
    this.#client.terminate();
  }
}
