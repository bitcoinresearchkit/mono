/** @param {HTMLCanvasElement} canvas */
export function createRenderer(canvas) {
  const context = canvas.getContext("2d");
  if (!context) throw "Expected context from canvas";
  let width = 0;
  let height = 0;
  let imageData = new ImageData(1, 1);
  let buffer = new Uint32Array();
  const emptyGeometry = {
    cols: -1,
    rows: -1,
    colX: new Int32Array(0),
    rowOffset: new Int32Array(0),
  };
  let geometry = { ...emptyGeometry };

  /**
   * @param {number} cols
   * @param {number} rows
   */
  function getGeometry(cols, rows) {
    if (geometry.cols === cols && geometry.rows === rows) return geometry;

    const colX = new Int32Array(cols + 1);
    for (let c = 0; c <= cols; c++) {
      colX[c] = ((c * width) / cols + 0.5) | 0;
    }

    const rowOffset = new Int32Array(rows + 1);
    for (let r = 0; r <= rows; r++) {
      rowOffset[r] = (((r * height) / rows + 0.5) | 0) * width;
    }

    geometry = { cols, rows, colX, rowOffset };
    return geometry;
  }

  /**
   * @param {number} col
   * @param {number} rows
   * @param {number} x0
   * @param {number} x1
   * @param {Int32Array} rowOffset
   * @param {(col: number, row: number) => number} getColor
   */
  function paintColumn(col, rows, x0, x1, rowOffset, getColor) {
    if (x0 === x1) return false;

    for (let r = 0; r < rows; r++) {
      const color = getColor(col, r);
      for (let off = rowOffset[r]; off < rowOffset[r + 1]; off += width) {
        buffer.fill(color, off + x0, off + x1);
      }
    }

    return true;
  }

  return {
    /**
     * @param {number} w
     * @param {number} h
     * @returns {boolean} whether the canvas was actually resized (true) or not (false)
     */
    resize(w, h) {
      const bound = canvas.getBoundingClientRect();
      const nextWidth = Math.floor(Math.min(w, bound.width));
      const nextHeight = Math.floor(Math.min(h, bound.height));
      if (nextWidth < 1 || nextHeight < 1) return false;
      if (nextWidth === width && nextHeight === height) return false;
      width = nextWidth;
      height = nextHeight;
      canvas.width = width;
      canvas.height = height;
      geometry = { ...emptyGeometry };
      imageData = context.createImageData(width, height);
      buffer = new Uint32Array(imageData.data.buffer);
      return true;
    },
    get width() {
      return width;
    },
    get height() {
      return height;
    },
    /**
     * Paint all cells or only dirty columns.
     * @param {number} cols
     * @param {number} rows
     * @param {(col: number, row: number) => number} getColor - returns ABGR uint32
     * @param {Iterable<number>} [dirty]
     */
    paint(cols, rows, getColor, dirty) {
      if (cols < 1 || rows < 1 || width < 1 || height < 1) return;

      const { colX, rowOffset } = getGeometry(cols, rows);

      if (dirty) {
        for (const c of dirty) {
          if (c < 0 || c >= cols) continue;
          const x0 = colX[c];
          const x1 = colX[c + 1];
          if (paintColumn(c, rows, x0, x1, rowOffset, getColor)) {
            context.putImageData(imageData, 0, 0, x0, 0, x1 - x0, height);
          }
        }
        return;
      }

      for (let c = 0; c < cols; c++) {
        paintColumn(c, rows, colX[c], colX[c + 1], rowOffset, getColor);
      }
      context.putImageData(imageData, 0, 0);
    },
  };
}
