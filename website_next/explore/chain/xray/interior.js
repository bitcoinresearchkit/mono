import { createXrayVolume } from "./volume.js";

const SVG_NS = "http://www.w3.org/2000/svg";
const VIEWBOX_WIDTH = 173.205;
const VIEWBOX_HEIGHT = 200;

export function createXrayInterior() {
  const svg = document.createElementNS(SVG_NS, "svg");

  svg.dataset.cubeXrayInterior = "";
  svg.setAttribute("aria-hidden", "true");
  svg.setAttribute("preserveAspectRatio", "none");
  svg.setAttribute("viewBox", `0 0 ${VIEWBOX_WIDTH} ${VIEWBOX_HEIGHT}`);

  return svg;
}

/**
 * @param {SVGSVGElement} svg
 * @param {import("../../block/preview/data.js").BlockPreviewData} data
 */
export function renderXrayInterior(svg, data) {
  const volume = createXrayVolume(data);
  const cubes = volume.cubes.sort(compareDepth);
  let faces = "";

  svg.style.setProperty("--xray-fill", String(volume.fill));

  for (const cube of cubes) {
    faces += createTransactionFaces(cube, volume.grid);
  }

  svg.innerHTML = faces;
}

/**
 * @param {{ east: number, north: number, up: number }} a
 * @param {{ east: number, north: number, up: number }} b
 */
function compareDepth(a, b) {
  return depth(a) - depth(b);
}

/** @param {{ east: number, north: number, up: number }} cube */
function depth(cube) {
  return cube.up - cube.east - cube.north;
}

/**
 * @param {{ color: string, east: number, faces: { left: boolean, right: boolean, top: boolean }, north: number, span: number, up: number }} cube
 * @param {number} grid
 */
function createTransactionFaces(cube, grid) {
  const east0 = cube.east / grid;
  const north0 = cube.north / grid;
  const up0 = cube.up / grid;
  const east1 = (cube.east + cube.span) / grid;
  const north1 = (cube.north + cube.span) / grid;
  const up1 = (cube.up + cube.span) / grid;
  let faces = "";

  if (cube.faces.left) {
    faces += createFace(
      [
        project(east0, north1, up0),
        project(east0, north0, up0),
        project(east0, north0, up1),
        project(east0, north1, up1),
      ],
      `color-mix(in oklch, ${cube.color} 72%, var(--black))`,
      "left",
    );
  }

  if (cube.faces.right) {
    faces += createFace(
      [
        project(east0, north0, up0),
        project(east1, north0, up0),
        project(east1, north0, up1),
        project(east0, north0, up1),
      ],
      `color-mix(in oklch, ${cube.color} 58%, var(--black))`,
      "right",
    );
  }

  if (cube.faces.top) {
    faces += createFace(
      [
        project(east0, north0, up1),
        project(east1, north0, up1),
        project(east1, north1, up1),
        project(east0, north1, up1),
      ],
      cube.color,
      "top",
    );
  }

  return faces;
}

/** @param {number} east @param {number} north @param {number} up */
function project(east, north, up) {
  return {
    x: VIEWBOX_WIDTH / 2 + (VIEWBOX_WIDTH / 2) * (east - north),
    y: VIEWBOX_HEIGHT * (1 - 0.25 * east - 0.25 * north - 0.5 * up),
  };
}

/**
 * @param {{ x: number, y: number }[]} points
 * @param {string} color
 * @param {string} side
 */
function createFace(points, color, side) {
  const coordinates = points.map(({ x, y }) => `${x},${y}`).join(" ");

  return `<polygon data-xray-prism-face="${side}" points="${coordinates}" style="--xray-fee-color:${color}"></polygon>`;
}
