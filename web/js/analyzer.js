// Entropy math. The one module that knows how a logged `p` turns into a cost, so
// it is also the seam where a wasm build drops in later.
//
// The coder reads `p` as P(bit == 1) on a 16-bit scale, and lerp() treats p == 0 as
// 2^-32 so there is always a chance left. The cost of a bit is the code length of the
// bit that was actually coded: -log2(P(actual)). Summed over book1 this comes to
// 3_602_891 bits = 450_361 bytes, which is exactly what the coder emitted.

const SCALE = 65536;

// Lookup beats Math.log2 by a wide margin when summing millions of bits.
const COST1 = new Float64Array(SCALE);
const COST0 = new Float64Array(SCALE);
for (let p = 0; p < SCALE; p++) {
  const q = p === 0 ? 2 ** -32 : p / SCALE;
  COST1[p] = -Math.log2(q);
  COST0[p] = -Math.log2(1 - q);
}

/** Cost in bits of coding `bit` under a 16-bit probability `p`. */
export function costBits(p, bit) {
  return bit ? COST1[p] : COST0[p];
}

/** `p` as a plain probability of the bit being 1. */
export function prob(p) {
  return p === 0 ? 2 ** -32 : p / SCALE;
}

/** The bit at absolute bit position `pos` of a byte array. */
export function bitAt(bytes, pos) {
  return (bytes[pos >> 3] >> (7 - (pos & 7))) & 1;
}

/**
 * Cost of byte `i` in bits, or -1 when any of its 8 bits has not been scanned yet.
 */
export function charCost(probs, nProbs, bytes, i) {
  const base = i << 3;
  if (base + 8 > nProbs) return -1;
  let sum = 0;
  for (let k = 0; k < 8; k++) {
    sum += bitAt(bytes, base + k) ? COST1[probs[base + k]] : COST0[probs[base + k]];
  }
  return sum;
}

/** Cost of bits [from, to) in bits. */
export function costRange(probs, from, to, bytes) {
  let sum = 0;
  for (let i = from; i < to; i++) {
    sum += bitAt(bytes, i) ? COST1[probs[i]] : COST0[probs[i]];
  }
  return sum;
}

/** Grow a typed array to hold at least `need` elements, keeping its contents. */
export function grow(arr, need) {
  if (arr.length >= need) return arr;
  const next = new arr.constructor(Math.max(need, Math.ceil(arr.length * 1.5)));
  next.set(arr);
  return next;
}

export const round3 = (x) => Math.round(x * 1000) / 1000;
export const fixed3 = (x) => x.toFixed(3);
export const commas = (n) => n.toLocaleString('en-US');
