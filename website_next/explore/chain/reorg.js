/** @typedef {import("../../modules/bitview-client/index.js").BlockInfoV1} Block */

/**
 * @param {Block[]} blocks
 * @param {(height: number) => HTMLButtonElement | null} findCube
 */
export function hasReorganized(blocks, findCube) {
  return blocks.some((block) => {
    const cube = findCube(block.height);

    return cube?.dataset.hash !== undefined && cube.dataset.hash !== block.id;
  });
}
