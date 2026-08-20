/**
 * HTML cube generator. Populates a .cube element with 15 face divs
 * styled in explorer.css. Uses pure CSS transforms (no SVG); the
 * earlier SVG-based implementation broke in Safari due to its
 * long-standing bugs around SVG transforms on <foreignObject>.
 *
 * Face order = z-order:
 *   3× .glass  rear   — translucent glass back faces
 *   3× .liquid rear   — opaque liquid backing (hidden at fill 0)
 *   3× .liquid front  — opaque liquid front (the visible 3 faces)
 *   3× .glass  front  — translucent glass front
 *   3× .face-text     — text overlays (top / right / left)
 *
 * @param {HTMLElement} cube
 * @param {number} [fill]
 * @returns {{ topFace: HTMLDivElement, rightFace: HTMLDivElement, leftFace: HTMLDivElement }}
 */
export function createCube(cube, fill = 1) {
  cube.style.setProperty("--fill", String(fill));

  /** @param {...string} cls */
  const face = (...cls) => {
    const d = document.createElement("div");
    d.className = `face ${cls.join(" ")}`;
    return /** @type {HTMLDivElement} */ (d);
  };

  const topFace   = face("face-text", "top");
  const rightFace = face("face-text", "right");
  const leftFace  = face("face-text", "left");

  cube.append(
    face("glass",  "bottom"),
    face("glass",  "rear-right"),
    face("glass",  "rear-left"),
    face("liquid", "bottom"),
    face("liquid", "rear-right"),
    face("liquid", "rear-left"),
    face("liquid", "right"),
    face("liquid", "left"),
    face("liquid", "top"),
    face("glass",  "right"),
    face("glass",  "left"),
    face("glass",  "top"),
    rightFace,
    leftFace,
    topFace,
  );

  return { topFace, rightFace, leftFace };
}
