import { CUBE_FACE_SPECS, createCubeFace } from "../../../cube/faces.js";

/**
 * @param {number} [fill]
 */
export function createCubeButton(fill = 1) {
  const element = document.createElement("button");

  element.type = "button";

  return { element, ...populateCube(element, fill) };
}

/**
 * @param {number} [fill]
 */
export function createCubeDiv(fill = 1) {
  const element = document.createElement("div");

  return { element, ...populateCube(element, fill) };
}

/**
 * @param {HTMLElement} element
 * @param {number} fill
 */
function populateCube(element, fill) {
  const visual = document.createElement("div");
  const topFace = createCubeFace("div", "text", "top");
  const rightFace = createCubeFace("div", "text", "right");
  const leftFace = createCubeFace("div", "text", "left");

  element.dataset.cube = "block";
  element.style.setProperty("--fill", String(fill));
  visual.dataset.cubeVisual = "";
  visual.append(
    ...CUBE_FACE_SPECS.map(([role, side]) =>
      createCubeFace("div", role, side),
    ),
    rightFace,
    leftFace,
    topFace,
  );
  element.append(visual);

  return { topFace, rightFace, leftFace };
}
