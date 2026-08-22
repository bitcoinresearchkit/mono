/**
 * Convert period ID to readable name
 * @param {string} id
 * @param {boolean} [compoundAdjective]
 */
export function periodIdToName(id, compoundAdjective) {
  const num = parseInt(id);
  const s = compoundAdjective || num === 1 ? "" : "s";
  switch (id.slice(-1)) {
    case "h":
      return `${num} Hour${s}`;
    case "d":
      return `${num} Day${s}`;
    case "w":
      return `${num} Week${s}`;
    case "m":
      return `${num} Month${s}`;
    case "y":
      return `${num} Year${s}`;
    default:
      return id;
  }
}
