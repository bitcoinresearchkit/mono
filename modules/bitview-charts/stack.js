// @ts-check
// With `percent`, a negative member counts as none: a share of a whole.

/**
 * @param {Float64Array[]} members @param {{ percent?: boolean }} [options]
 * @returns {{ tops: Float64Array[], feet: Float64Array[], scale: Float64Array | null }}
 */
export function stack(members, { percent = false } = {}) {
  const n = members.reduce((most, member) => Math.max(most, member.length), 0), total = new Float64Array(n);
  const tops = members.map(() => new Float64Array(n)), feet = members.map(() => new Float64Array(n));
  for (let k = 0; k < members.length; k++) {
    const member = members[k], top = tops[k], foot = feet[k];
    for (let i = 0; i < n; i++) {
      foot[i] = total[i];
      const v = i < member.length ? member[i] : NaN;
      // (Finite by a subtraction: no call each value.)
      top[i] = v - v === 0 ? (total[i] += percent && v < 0 ? 0 : v) : NaN;
    }
  }
  if (!percent) return { tops, feet, scale: null };
  // (A typed array's own map: on a phone, faster than a loop written out.)
  const scale = total.map((sum) => (sum ? 100 / sum : NaN));
  for (let k = 0; k < members.length; k++) {
    const top = tops[k], foot = feet[k];
    for (let i = 0; i < n; i++) (top[i] *= scale[i]), (foot[i] *= scale[i]);
  }
  return { tops, feet, scale };
}
