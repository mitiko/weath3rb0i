// Global display settings. CR lives here rather than on a model because two models being
// compared have to share one colour scale.

import { emit } from './bus.js';

export const view = {
  cr: 1,             // bits per bit at the middle of the ramp; 1 is no compression at all
  measured: 0,       // the ratio the files actually came to, kept so CR can restore it
  pinned: false,     // the user typed a CR, so stop measuring over them
  filter: null,      // [lo, hi] bucket band, null means the whole ramp
  anchor: null,      // bit position, held until another is picked
  hover: null,       // bit position under the pointer, transient
  tab: 'source',     // which dock tab is showing
};

export function setCR(cr, pinned = false) {
  view.cr = cr;
  view.pinned = pinned;
  emit('cr');
}

/** The ratio measured off the files. Recorded even while pinned, so CR can put it back. */
export function setMeasured(cr) {
  view.measured = cr;
  if (!view.pinned) setCR(cr);
}

export function setFilter(band) {
  view.filter = band;
  emit('filter');
}

export function setAnchor(pos) {
  view.anchor = pos;
  emit('anchor');
}

export function setHover(pos) {
  view.hover = pos;
  emit('hover');
}

export function setTab(tab) {
  view.tab = tab;
  emit('tab');
}
