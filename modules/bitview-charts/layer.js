// @ts-check
// A canvas the chart draws on, laid in its element.

/**
 * A picture, so out of what's read: the plot's words are its legends' and the page's to say.
 * @param {string} css its style, placed by the chart
 */
export function layer(css) {
  const canvas = document.createElement("canvas");
  canvas.style.cssText = css;
  canvas.ariaHidden = "true";
  return canvas;
}
