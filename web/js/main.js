// Three independent inputs, any order:
//
//   source   the data that was compressed. Enough on its own for the text grid and the
//            context stats, which need no model output at all
//   .p16     one u16 per bit, the probability it was coded with, drives every colour
//   .jsonl   the model's state per bit, only the encoder state panel needs it (optional)
//
// Everything past this file talks over the bus in core/bus.js. What is left here is the
// pickers, the landing card, and the one rule that ties the two files to the colour scale.

import { initColors } from './color.js';
import { on } from './core/bus.js';
import { model } from './core/model.js';
import { loadSource, source } from './core/source.js';
import { JsonlMachine } from './core/time-machine.js';
import { setMeasured } from './core/view.js';
import './components/index.js';

const el = (id) => document.getElementById(id);

initColors();

pick('pick-source', (file) => loadSource(file));
pick('pick-probs', (file) => model.load(file));
pick('pick-jsonl', (file) => {
  model.state?.close();
  model.state = new JsonlMachine(file);
});

// reveal first: the grid sizes itself against the layout, which is wrong while it is hidden
function pick(id, load) {
  el(id).onchange = (e) => {
    const file = e.target.files[0];
    if (!file) return;
    reveal();
    load(file);
  };
}

// The colour midpoint is the file's compression ratio, so yellow is an average prediction.
// Measured once both files are in: a ratio against a partial sum drifts every chunk.
const measure = () => {
  if (source.complete && model.complete && source.bytes.length) {
    setMeasured(model.entropy / (source.bytes.length * 8));
  }
};
on('source:done', measure);
on('model:done', measure);

/** First file picked reveals the app and moves the pickers into the dock. */
function reveal() {
  if (el('landing').hidden) return;
  el('files-slot').append(el('pickers'));
  el('landing').hidden = true;
  document.body.classList.add('loaded');
  for (const id of ['bar', 'main', 'dock', 'scanbar']) el(id).hidden = false;
  dispatchEvent(new Event('resize')); // everything just moved, so let the grid re-measure
}
