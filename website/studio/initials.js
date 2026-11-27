// Spaces keep separate pieces from matching across their boundaries.
const WORDS = /[a-z0-9]+/gi;

/** @param {string[]} pieces */
export function initialsOf(pieces) {
  return pieces.map((piece) => (piece.toLowerCase().match(WORDS) ?? []).map((word) => word[0]).join("")).join(" ");
}

/** @param {string} word */
export const initialsWord = (word) => /^[a-z]{2,}$/.test(word);

/** @param {string} text @param {string[]} typed @returns {[number, number][]} */
export function initialsRanges(text, typed) {
  const words = [...text.matchAll(WORDS)], initials = words.map(([word]) => word[0].toLowerCase()).join(""), ranges = /** @type {[number, number][]} */ ([]);
  for (const word of typed) {
    if (!initialsWord(word)) continue;
    for (let at = initials.indexOf(word); at >= 0; at = initials.indexOf(word, at + 1)) {
      for (let i = at; i < at + word.length; i++) ranges.push([words[i].index, words[i].index + words[i][0].length]);
    }
  }
  return ranges;
}
