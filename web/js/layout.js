// Bytes -> rows of an 80-column grid. Computed here rather than left to the browser so
// the virtual scroller knows exactly which bytes are on which row.
//
// A row breaks when the next glyph would not fit, or right after a newline. Rows are
// therefore ragged and the text reads like prose.

const COLS = 80;

const ESCAPES = new Map([
  [0x0a, '\\n'],
  [0x0d, '\\r'],
  [0x09, '\\t'],
  [0x00, '\\0'],
]);

const HEX = '0123456789abcdef';

/** How the byte is drawn, and how many columns it takes. */
export function glyph(b) {
  if (b >= 0x20 && b <= 0x7e) return { text: String.fromCharCode(b), cols: 1 };
  const esc = ESCAPES.get(b);
  if (esc) return { text: esc, cols: 2 };
  return { text: '\\x' + HEX[b >> 4] + HEX[b & 15], cols: 4 };
}

export function width(b) {
  if (b >= 0x20 && b <= 0x7e) return 1;
  return ESCAPES.has(b) ? 2 : 4;
}

export class Layout {
  constructor(cols = COLS) {
    this.cols = cols;
    this.starts = [0]; // byte index each row begins at
    this.done = 0;     // bytes consumed
  }

  /**
   * Extend the layout to cover bytes [0, len). The last row is recomputed because more
   * data may have arrived to fill it.
   */
  extend(bytes, len) {
    if (len <= this.done) return;
    let from = this.starts[this.starts.length - 1];
    let col = 0;
    for (let i = from; i < this.done; i++) col += width(bytes[i]);

    for (let i = this.done; i < len; i++) {
      const b = bytes[i];
      const w = width(b);
      if (col + w > this.cols) {
        this.starts.push(i);
        col = 0;
      }
      col += w;
      if (b === 0x0a) {
        // always break, even at the tail: the row stays empty until more bytes arrive
        this.starts.push(i + 1);
        col = 0;
      }
    }
    this.done = len;
  }

  get rows() {
    return this.starts.length;
  }

  /** Byte range [start, end) of a row. */
  range(row) {
    return [this.starts[row], row + 1 < this.starts.length ? this.starts[row + 1] : this.done];
  }

  /** The row containing byte `i`. */
  rowOf(i) {
    const s = this.starts;
    let lo = 0, hi = s.length - 1;
    while (lo < hi) {
      const mid = (lo + hi + 1) >> 1;
      if (s[mid] <= i) lo = mid; else hi = mid - 1;
    }
    return lo;
  }
}
