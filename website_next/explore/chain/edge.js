import { onPlainClick } from "./events.js";

/**
 * @param {"tip"} name
 * @param {string} verticalLabel
 * @param {string} horizontalLabel
 * @param {string} title
 * @param {() => void} handler
 */
export function createEdgeButton(
  name,
  verticalLabel,
  horizontalLabel,
  title,
  handler,
) {
  const button = document.createElement("button");

  button.type = "button";
  button.title = title;
  button.ariaLabel = title;
  button.dataset.edge = name;
  button.dataset.horizontalLabel = horizontalLabel;
  button.textContent = verticalLabel;
  onPlainClick(button, handler);

  return button;
}
