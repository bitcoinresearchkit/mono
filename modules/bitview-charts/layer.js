// @ts-check

/**
 * Out of what's read: the plot's words are its legends' and the page's to say.
 * @param {string} css
 */
export function layer(css) {
  const canvas = document.createElement("canvas");
  canvas.style.cssText = css;
  canvas.ariaHidden = "true";
  return canvas;
}
