// Exact byte-sequence search over the source.
//
// The needle is bytes, not text, because that is what the file is. Typing goes through
// TextEncoder; a selection in the grid hands over the bytes it covers directly, so a
// non-UTF-8 run still searches for exactly what is on screen.

import { emit, on } from './bus.js';
import { source } from './source.js';

export const search = {
  text: '',                    // what the box shows
  needle: new Uint8Array(0),   // what we look for
  matches: [],                 // byte offset of each hit
  at: -1,                      // index into matches, -1 when there are none
};

export function setQuery(text, needle) {
  search.text = text;
  search.needle = needle || new TextEncoder().encode(text);
  run();
}

export function clear() {
  setQuery('', new Uint8Array(0));
}

/** Move to the next or previous hit, wrapping. The only thing that scrolls the view. */
export function step(delta) {
  const n = search.matches.length;
  if (!n) return;
  // a fresh query has no current hit, so the first step lands on an end rather than past it
  search.at = search.at < 0 ? (delta > 0 ? 0 : n - 1) : (search.at + delta + n) % n;
  emit('search');
  emit('search:goto');
}

/** 0 for no hit, 1 inside a hit, 2 inside the current one. */
export function hitAt(i) {
  const m = search.needle.length;
  if (!m || !search.matches.length) return 0;
  const a = search.matches;
  let lo = 0, hi = a.length - 1, found = -1;
  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    if (a[mid] <= i) { found = mid; lo = mid + 1; } else hi = mid - 1;
  }
  if (found < 0 || i >= a[found] + m) return 0;
  return found === search.at ? 2 : 1;
}

function run() {
  const m = search.needle.length;
  const n = source.layout.done;
  const bytes = source.bytes;
  const hits = [];

  outer:
  for (let i = 0; m && i + m <= n; i++) {
    if (bytes[i] !== search.needle[0]) continue;
    for (let j = 1; j < m; j++) if (bytes[i + j] !== search.needle[j]) continue outer;
    hits.push(i);
  }

  search.matches = hits;
  search.at = -1;
  emit('search');
}

// a query typed while the file is still streaming has only seen part of it
on('source:done', () => { if (search.needle.length) run(); });
