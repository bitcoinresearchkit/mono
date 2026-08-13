import { MAX_BLOCK_WEIGHT } from "../../block/format.js";
import { createXrayTransactions } from "./sample.js";

const CALIBRATION_STEPS = 24;
const GRID = 48;
const SUPPORT_LOOKAHEAD = 4;

/** @param {import("../../block/preview/data.js").BlockPreviewData} data */
export function createXrayVolume(data) {
  const fill = Math.min(1, data.blockWeight / MAX_BLOCK_WEIGHT);
  const layers = Math.max(1, Math.round(fill * GRID));
  const capacity = GRID * GRID * layers;
  const transactions = createXrayTransactions(data);
  const spans = resolveSpans(transactions, capacity, layers);
  const occupied = new Uint8Array(capacity);
  const cursors = new Uint32Array(layers + 1);
  const placed = transactions.map((transaction, index) => {
    const placement = placeCube(
      occupied,
      cursors,
      layers,
      spans[index],
    );

    return { ...transaction, ...placement };
  });
  const cubes = [];

  for (const cube of placed) {
    const faces = findVisibleFaces(occupied, layers, cube);

    if (faces.left || faces.right || faces.top) {
      cubes.push({ ...cube, faces });
    }
  }

  return { cubes, fill: layers / GRID, grid: GRID };
}

/**
 * @param {{ weight: number }[]} transactions
 * @param {number} capacity
 * @param {number} layers
 */
function resolveSpans(transactions, capacity, layers) {
  const totalWeight = transactions.reduce((sum, { weight }) => sum + weight, 0);
  const ideals = Float64Array.from(
    transactions,
    ({ weight }) => Math.cbrt((weight * capacity) / totalWeight),
  );
  let lowerScale = 0;
  let upperScale = 1;

  while (scaledVolume(ideals, upperScale, layers) <= capacity) {
    upperScale *= 2;
  }

  for (let step = 0; step < CALIBRATION_STEPS; step += 1) {
    const scale = (lowerScale + upperScale) / 2;

    if (scaledVolume(ideals, scale, layers) <= capacity) {
      lowerScale = scale;
    } else {
      upperScale = scale;
    }
  }

  const spans = Uint8Array.from(ideals, (ideal) => {
    return resolveSpan(ideal, lowerScale, layers);
  });
  const candidates = Uint32Array.from(spans, (_, index) => index);
  let volume = spans.reduce((sum, span) => sum + span ** 3, 0);

  candidates.sort((a, b) => {
    return (spans[a] + 0.5) / ideals[a] -
        (spans[b] + 0.5) / ideals[b] || a - b;
  });

  for (const index of candidates) {
    const span = spans[index];
    if (span === layers) continue;

    const growth = (span + 1) ** 3 - span ** 3;
    if (volume + growth > capacity) continue;

    spans[index] += 1;
    volume += growth;
  }

  return spans;
}

/** @param {Float64Array} ideals @param {number} scale @param {number} layers */
function scaledVolume(ideals, scale, layers) {
  let volume = 0;

  for (const ideal of ideals) {
    volume += resolveSpan(ideal, scale, layers) ** 3;
  }

  return volume;
}

/** @param {number} ideal @param {number} scale @param {number} layers */
function resolveSpan(ideal, scale, layers) {
  return Math.max(1, Math.min(layers, Math.round(ideal * scale)));
}

/**
 * @param {Uint8Array} occupied
 * @param {Uint32Array} cursors
 * @param {number} layers
 * @param {number} span
 */
function placeCube(occupied, cursors, layers, span) {
  let resolvedSpan = span;
  let placement = findPosition(
    occupied,
    layers,
    resolvedSpan,
    cursors[resolvedSpan],
  );

  while (placement === null) {
    resolvedSpan -= 1;
    placement = findPosition(
      occupied,
      layers,
      resolvedSpan,
      cursors[resolvedSpan],
    );
  }

  fillCells(
    occupied,
    placement.east,
    placement.north,
    placement.up,
    resolvedSpan,
  );
  cursors[resolvedSpan] = placement.index + resolvedSpan;

  return { ...placement, span: resolvedSpan };
}

/**
 * @param {Uint8Array} occupied
 * @param {number} layers
 * @param {number} span
 * @param {number} start
 */
function findPosition(occupied, layers, span, start) {
  const layerArea = GRID * GRID;
  const lastUp = layers - span;
  const lastNorth = GRID - span;
  const lastEast = GRID - span;
  const startUp = Math.floor(start / layerArea);
  const startInLayer = start % layerArea;
  const startNorth = Math.floor(startInLayer / GRID);
  const startEast = startInLayer % GRID;
  for (let up = startUp; up <= lastUp; up += 1) {
    const firstNorth = up === startUp ? startNorth : 0;
    let best = /** @type {{ east: number, index: number, north: number, support: number, up: number } | null} */ (
      null
    );
    let remaining = SUPPORT_LOOKAHEAD;

    for (let north = firstNorth; north <= lastNorth; north += 1) {
      const firstEast = up === startUp && north === startNorth
        ? startEast
        : 0;

      for (let east = firstEast; east <= lastEast; east += 1) {
        if (canPlace(occupied, east, north, up, span)) {
          const placement = {
            east,
            index: up * layerArea + north * GRID + east,
            north,
            support: supportBelow(occupied, east, north, up, span),
            up,
          };

          if (placement.support === span ** 2) return placement;
          if (best === null || placement.support > best.support) best = placement;
        }

        if (best !== null && --remaining === 0) return best;
      }
    }

    if (best !== null) return best;
  }

  return null;
}

/**
 * @param {Uint8Array} occupied
 * @param {number} east
 * @param {number} north
 * @param {number} up
 * @param {number} span
 */
function supportBelow(occupied, east, north, up, span) {
  if (up === 0) return span ** 2;

  let support = 0;

  for (let y = north; y < north + span; y += 1) {
    for (let x = east; x < east + span; x += 1) {
      support += occupied[indexOf(x, y, up - 1)];
    }
  }

  return support;
}

/**
 * @param {Uint8Array} occupied
 * @param {number} east
 * @param {number} north
 * @param {number} up
 * @param {number} span
 */
function canPlace(occupied, east, north, up, span) {
  for (let z = up; z < up + span; z += 1) {
    for (let y = north; y < north + span; y += 1) {
      const offset = indexOf(east, y, z);

      for (let x = 0; x < span; x += 1) {
        if (occupied[offset + x]) return false;
      }
    }
  }

  return true;
}

/**
 * @param {Uint8Array} occupied
 * @param {number} east
 * @param {number} north
 * @param {number} up
 * @param {number} span
 */
function fillCells(occupied, east, north, up, span) {
  for (let z = up; z < up + span; z += 1) {
    for (let y = north; y < north + span; y += 1) {
      const offset = indexOf(east, y, z);

      occupied.fill(1, offset, offset + span);
    }
  }
}

/**
 * @param {Uint8Array} occupied
 * @param {number} layers
 * @param {{ east: number, north: number, span: number, up: number }} cube
 */
function findVisibleFaces(occupied, layers, cube) {
  return {
    left: hasVisibleLeft(occupied, cube),
    right: hasVisibleRight(occupied, cube),
    top: hasVisibleTop(occupied, layers, cube),
  };
}

/**
 * @param {Uint8Array} occupied
 * @param {{ east: number, north: number, span: number, up: number }} cube
 */
function hasVisibleLeft(occupied, cube) {
  if (cube.east === 0) return true;

  for (let z = cube.up; z < cube.up + cube.span; z += 1) {
    for (let y = cube.north; y < cube.north + cube.span; y += 1) {
      if (!occupied[indexOf(cube.east - 1, y, z)]) return true;
    }
  }

  return false;
}

/**
 * @param {Uint8Array} occupied
 * @param {{ east: number, north: number, span: number, up: number }} cube
 */
function hasVisibleRight(occupied, cube) {
  if (cube.north === 0) return true;

  for (let z = cube.up; z < cube.up + cube.span; z += 1) {
    for (let x = cube.east; x < cube.east + cube.span; x += 1) {
      if (!occupied[indexOf(x, cube.north - 1, z)]) return true;
    }
  }

  return false;
}

/**
 * @param {Uint8Array} occupied
 * @param {number} layers
 * @param {{ east: number, north: number, span: number, up: number }} cube
 */
function hasVisibleTop(occupied, layers, cube) {
  const up = cube.up + cube.span;

  if (up === layers) return true;

  for (let y = cube.north; y < cube.north + cube.span; y += 1) {
    for (let x = cube.east; x < cube.east + cube.span; x += 1) {
      if (!occupied[indexOf(x, y, up)]) return true;
    }
  }

  return false;
}

/** @param {number} east @param {number} north @param {number} up */
function indexOf(east, north, up) {
  return up * GRID * GRID + north * GRID + east;
}
