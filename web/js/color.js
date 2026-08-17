// Cost in bits -> one of BUCKETS background colours.
//
// 43% of book1 sits above the baseline, so the middle of the ramp has to stay pale or half
// the page shouts. Chroma only climbs for the tail, which is what makes outliers findable.

import { on } from './core/bus.js';
import { view } from './core/view.js';

export const BUCKETS = 32;

// buckets outside the selected band are painted this, which is how the grid narrows to
// just the predictable or just the random end
const MUTED = '#eeeef1';

// Spaced so blue, green, yellow, orange and red get about six buckets each, and picked to
// stay inside sRGB: light blue and light orange both hold very little chroma.
const ANCHORS = [
  { t: 0.00, L: 0.915, C: 0.042, h: 245 }, // free
  { t: 0.14, L: 0.920, C: 0.052, h: 222 }, // very predictable
  { t: 0.30, L: 0.925, C: 0.105, h: 150 }, // well predicted
  { t: 0.50, L: 0.935, C: 0.098, h: 100 }, // baseline
  { t: 0.70, L: 0.815, C: 0.100, h: 52 },  // no better than raw
  { t: 1.00, L: 0.620, C: 0.200, h: 22 },  // worst
];

// Cost in bits to badness, scaled by the baseline, with 8 bits kept as a fixed landmark.
// The max() calls keep the stops ordered for any baseline.
function stops(baseline) {
  const b = baseline;
  return [
    [0, 0.0],
    [b / 2, 0.35],
    [b, 0.5],
    [Math.max(8, b * 1.4), 0.7],
    [Math.max(12, b * 2.2), 0.9],
    [Math.max(16, b * 3.0), 1.0],
  ];
}

let curve = stops(8);

/** Badness in [0, 1] for a character cost in bits. */
export function badness(cost) {
  const last = curve.length - 1;
  if (cost <= 0) return 0;
  if (cost >= curve[last][0]) return 1;
  for (let i = 1; i <= last; i++) {
    const [c1, t1] = curve[i];
    if (cost <= c1) {
      const [c0, t0] = curve[i - 1];
      return t0 + ((cost - c0) / (c1 - c0)) * (t1 - t0);
    }
  }
  return 1;
}

/** Bucket index for a character cost in bits. */
export function bucket(cost) {
  return Math.round(badness(cost) * (BUCKETS - 1));
}

/** Inverse of badness: the cost in bits that maps to `t`. */
export function costFor(t) {
  const last = curve.length - 1;
  if (t <= 0) return 0;
  if (t >= 1) return curve[last][0];
  for (let i = 1; i <= last; i++) {
    const [c1, t1] = curve[i];
    if (t <= t1) {
      const [c0, t0] = curve[i - 1];
      return c0 + ((t - t0) / (t1 - t0)) * (c1 - c0);
    }
  }
  return curve[last][0];
}

/** Bucket index for a single bit's cost: same curve, scaled to a per-bit baseline. */
export function bitBucket(cost) {
  return bucket(cost * 8);
}

function colorAt(t) {
  let i = 1;
  while (i < ANCHORS.length - 1 && t > ANCHORS[i].t) i++;
  const a = ANCHORS[i - 1], b = ANCHORS[i];
  const f = (t - a.t) / (b.t - a.t);
  const L = a.L + f * (b.L - a.L);
  const C = a.C + f * (b.C - a.C);
  const h = a.h + f * (b.h - a.h);
  return `oklch(${L.toFixed(3)} ${C.toFixed(3)} ${h.toFixed(1)})`;
}

/** The colour of a bucket, for legends and one-off elements. */
export function bucketColor(i) {
  return colorAt(i / (BUCKETS - 1));
}

let sheet = null;

// One stylesheet write recolours the text grid and the bit strip together, so a change of
// CR or filter costs no re-render anywhere. Only possible because we stay in light DOM.
function apply() {
  curve = stops(view.cr * 8);
  if (!sheet) {
    sheet = document.createElement('style');
    document.head.append(sheet);
  }
  const [lo, hi] = view.filter || [0, BUCKETS - 1];
  let css = '';
  for (let i = 0; i < BUCKETS; i++) {
    css += `.e${i}{background:${i >= lo && i <= hi ? bucketColor(i) : MUTED}}`;
  }
  sheet.textContent = css;
}

/** Build the 32 rules and keep them in step with the view. Call once at startup. */
export function initColors() {
  on('cr', apply);
  on('filter', apply);
  apply();
}
