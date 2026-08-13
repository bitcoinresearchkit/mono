import { createXrayInterior, renderXrayInterior } from "./interior.js";

export function createXrayRenderer() {
  const interior = createXrayInterior();
  let attachedCube = /** @type {HTMLButtonElement | null} */ (null);
  let renderedHash = "";

  /** @param {HTMLButtonElement} cube @param {string} hash */
  function attach(cube, hash) {
    const visual = cube.querySelector("[data-cube-visual]");
    const frontFace = visual?.querySelector(
      '[data-role="liquid"][data-side="right"]',
    );

    if (!visual || !frontFace) return false;

    attachedCube = cube;
    const ready = renderedHash === hash;

    visual.insertBefore(interior, frontFace);
    cube.dataset.xray = "";
    if (ready) cube.dataset.xrayReady = "";

    return ready;
  }

  /**
   * @param {string} hash
   * @param {import("../../block/preview/data.js").BlockPreviewData} data
   */
  function render(hash, data) {
    if (!attachedCube) return;

    renderXrayInterior(interior, data);
    renderedHash = hash;
    attachedCube.dataset.xrayReady = "";
  }

  /** @param {HTMLButtonElement} cube */
  function hide(cube) {
    cube.removeAttribute("data-xray");
    cube.removeAttribute("data-xray-ready");
  }

  /** @param {HTMLButtonElement} cube */
  function detach(cube) {
    if (attachedCube !== cube) return;

    hide(cube);
    interior.remove();
    attachedCube = null;
  }

  return /** @type {const} */ ({ attach, detach, hide, render });
}
