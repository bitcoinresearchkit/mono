/**
 * @param {number} [fill]
 */
export function createCubeAnchor(fill = 1) {
  const el = document.createElement("a");
  return { el, ...populateCube(el, fill) };
}

/**
 * @param {number} [fill]
 */
export function createCubeDiv(fill = 1) {
  const el = document.createElement("div");
  return { el, ...populateCube(el, fill) };
}

/**
 * @param {HTMLElement} el
 * @param {number} fill
 */
function populateCube(el, fill) {
  el.classList.add("cube");
  el.style.setProperty("--fill", String(fill));

  const topFace = createFace("face-text", "top");
  const rightFace = createFace("face-text", "right");
  const leftFace = createFace("face-text", "left");

  el.append(
    createFace("glass", "bottom"),
    createFace("glass", "rear-right"),
    createFace("glass", "rear-left"),
    createFace("liquid", "bottom"),
    createFace("liquid", "rear-right"),
    createFace("liquid", "rear-left"),
    createFace("liquid", "right"),
    createFace("liquid", "left"),
    createFace("liquid", "top"),
    createFace("glass", "right"),
    createFace("glass", "left"),
    createFace("glass", "top"),
    rightFace,
    leftFace,
    topFace,
  );

  return { topFace, rightFace, leftFace };
}

/**
 * @param {string} role
 * @param {string} position
 * */
function createFace(role, position) {
  const div = document.createElement("div");
  div.className = `face ${role} ${position}`;
  return div;
}
