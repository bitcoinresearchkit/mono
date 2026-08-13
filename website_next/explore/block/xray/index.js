import {
  createXrayInterior,
  renderXrayInterior,
} from "../../chain/xray/interior.js";

/**
 * @param {Promise<import("../preview/data.js").BlockPreviewData>} data
 * @param {HTMLElement} scrollRoot
 */
export function createBlockXrayPane(data, scrollRoot) {
  const element = document.createElement("section");
  const title = document.createElement("h2");
  const content = document.createElement("div");
  const status = document.createElement("p");
  const interior = createXrayInterior();
  let live = true;

  element.dataset.blockXray = "";
  title.textContent = "X-ray";
  status.textContent = "Loading";
  status.dataset.blockXrayStatus = "";
  content.append(status, interior);
  element.append(title, content);

  const observer = new IntersectionObserver(
    (entries) => {
      if (!entries.some(({ isIntersecting }) => isIntersecting)) return;

      observer.disconnect();
      void render();
    },
    { root: scrollRoot, rootMargin: "75% 0px" },
  );

  observer.observe(element);

  async function render() {
    try {
      const preview = await data;
      if (!live) return;

      renderXrayInterior(interior, preview);
      status.remove();
      element.dataset.ready = "";
    } catch (error) {
      if (!live) return;

      console.error("explore block x-ray:", error);
      status.textContent = "Unavailable";
    }
  }

  function destroy() {
    live = false;
    observer.disconnect();
    interior.replaceChildren();
  }

  return /** @type {const} */ ({ destroy, element });
}
