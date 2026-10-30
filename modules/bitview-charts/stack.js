// @ts-check
// A stack's edges from its members' columns, the first at the bottom: each edge the sum of the members up to it, or
// with `percent`, their share of all of them (0 to 100). A member with no value at a point counts as none there; a
// point where no member has one, none (the stack broken there). Bands between each two edges (the first's from
// zero) and lines along them draw the stack.

/**
 * @param {Float64Array[]} members @param {{ percent?: boolean }} [options]
 * @returns {Float64Array[]} an edge a member, each one's top
 */
export function stack(members, { percent = false } = {}) {
  const n = members.reduce((most, member) => Math.max(most, member.length), 0);
  const edges = members.map(() => new Float64Array(n).fill(NaN));
  for (let i = 0; i < n; i++) {
    let sum = 0, any = false;
    for (let k = 0; k < members.length; k++) {
      const v = i < members[k].length ? members[k][i] : NaN;
      if (v === v) (sum += v), (any = true);
    }
    if (!any || (percent && !sum)) continue;
    let at = 0;
    for (let k = 0; k < members.length; k++) {
      const v = i < members[k].length ? members[k][i] : NaN;
      if (v === v) at += v;
      edges[k][i] = percent ? (at / sum) * 100 : at;
    }
  }
  return edges;
}
