// Wiring: keep one shared state object that everything else reads, and start each file
// the moment it is picked. Three independent inputs, any order:
//
//   source      the compressed data. Enough on its own for the text grid and context
//               stats, which need no model output at all
//   .p16        one u16 per bit, the probability it was coded with, drives every colour
//   .jsonl      full model state per bit, only the state panel needs it (optional)

import { costRange, commas, fixed3 } from './analyzer.js';
import { initColors, setBaseline } from './color.js';
import { Layout } from './layout.js';
import { TextView } from './textview.js';
import { Inspector } from './inspector.js';
import { startMeters } from './meters.js';

const state = {
  source: { name: '', size: 0, sha256: '' },
  bytes: new Uint8Array(0),
  layout: new Layout(),
  probs: new Uint16Array(0),
  nProbs: 0,       // bits usable now: probability loaded and source byte present
  entropy: 0,      // sum of costs over those bits
  metadata: null,
  anchor: null,    // anchored bit position
};

const el = (id) => document.getElementById(id);

let worker = null;
let loadedProbs = 0;   // u16 entries read from the sidecar
let probsSize = 0;
let lineSeq = 0;
const linePending = new Map();

initColors(el('legend'));
const view = new TextView(state, {
  onHover: (i) => showPosition(i === null ? state.anchor : i * 8),
  onPick: pick,
});
const inspector = new Inspector(state, { getLine, onPick: pick });

el('pick-source').onchange = (e) => e.target.files[0] && loadSource(e.target.files[0]);
el('pick-probs').onchange = (e) => e.target.files[0] && loadProbs(e.target.files[0]);
el('pick-jsonl').onchange = (e) => e.target.files[0] && startIndex(e.target.files[0]);

el('baseline').onchange = (e) => {
  setBaseline(Math.max(0.001, +e.target.value || 0.586) * 8);
  view.invalidate();
  inspector.refresh();
};

/** First file picked reveals the app and moves the pickers into the dock. */
function reveal() {
  if (!el('landing').hidden) {
    el('files-slot').append(el('pickers'));
    el('landing').hidden = true;
    document.body.classList.add('loaded');
    el('bar').hidden = el('main').hidden = el('dock').hidden = el('scanbar').hidden = false;
    startMeters();
  }
}

/** Stream the source so the first rows appear well before the read finishes. */
async function loadSource(file) {
  reveal();
  state.source = { name: file.name, size: file.size, sha256: 'computing…' };
  state.bytes = new Uint8Array(file.size);
  state.layout = new Layout();
  state.nProbs = 0;
  state.entropy = 0;
  el('s-source').textContent = `${file.name} · ${commas(file.size)} B`;
  checkPair();

  await stream(file, el('src-progress'), (chunk, at) => {
    state.bytes.set(chunk, at);
    state.layout.extend(state.bytes, at + chunk.length);
    advance();
    view.invalidate();
  });

  const digest = await crypto.subtle.digest('SHA-256', state.bytes);
  state.source.sha256 = Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, '0')).join('');
  el('s-source').title = 'sha256 ' + state.source.sha256;
}

/** Little-endian u16 per bit. Every platform a browser runs on is little-endian. */
async function loadProbs(file) {
  reveal();
  const raw = new Uint8Array(file.size + (file.size & 1));
  state.probs = new Uint16Array(raw.buffer);
  loadedProbs = 0;
  probsSize = file.size;
  checkPair();

  await stream(file, el('probs-progress'), (chunk, at) => {
    raw.set(chunk, at);
    loadedProbs = (at + chunk.length) >> 1;
    advance();
  });
}

function startIndex(file) {
  reveal();
  inspector.hasState = true;
  if (worker) worker.terminate();
  worker = new Worker('js/index-worker.js', { type: 'module' });

  worker.onmessage = (e) => {
    const m = e.data;
    switch (m.type) {
      case 'meta':
        state.metadata = m.metadata;
        break;
      case 'progress':
        el('index-progress').value = m.bytesRead / m.total;
        break;
      case 'done':
        el('index-progress').value = 1;
        note(m.lines === state.bytes.length * 8 ? ''
          : `state file has ${commas(m.lines)} lines for ${commas(state.bytes.length * 8)} bits`);
        break;
      case 'line': {
        const resolve = linePending.get(m.id);
        if (resolve) { linePending.delete(m.id); resolve(m.json); }
        break;
      }
      case 'error':
        note('index failed: ' + m.message);
        break;
    }
  };
  worker.postMessage({ cmd: 'index', file });
}

async function stream(file, progress, onChunk) {
  const reader = file.stream().getReader();
  let at = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) { progress.value = 1; return; }
    onChunk(value, at);
    at += value.length;
    progress.value = at / file.size;
  }
}

/**
 * A bit is usable once its probability has loaded and the source byte holding it has too.
 * Either side can be behind, so both call this.
 */
function advance() {
  const limit = Math.min(loadedProbs, state.layout.done * 8);
  if (limit <= state.nProbs) return;
  state.entropy += costRange(state.probs, state.nProbs, limit, state.bytes);
  state.nProbs = limit;

  el('s-entropy').textContent = commas(Math.round(state.entropy / 8)) + ' B';
  el('s-ratio').textContent = fixed3(state.entropy / state.nProbs);
  view.invalidate();
  inspector.refresh();
}

/** Warn when the probability count cannot match the source. */
function checkPair() {
  if (!probsSize || !state.bytes.length) return;
  note(probsSize === state.bytes.length * 16 ? ''
    : `${commas(probsSize / 2)} probabilities for ${commas(state.bytes.length * 8)} bits, the files may not match`);
}

function note(msg) {
  el('scan-status').textContent = msg ? '⚠ ' + msg : '';
}

/** Data line `k` of the JSONL, which is the state entering bit `k`. */
function getLine(k) {
  if (!worker) return Promise.resolve(null);
  const id = ++lineSeq;
  return new Promise((resolve) => {
    linePending.set(id, resolve);
    worker.postMessage({ cmd: 'line', id, line: k });
  });
}

function pick(pos) {
  state.anchor = pos;
  showPosition(pos);
  view.markAnchor();
  inspector.show(pos);
}

/** Hover updates the readout only; a pick moves the anchor with it. */
function showPosition(pos) {
  if (pos === null) return;
  el('s-pos').textContent = commas(pos);
  el('s-byte').textContent = commas(pos >> 3);
  el('s-off').textContent = pos & 7;
}
