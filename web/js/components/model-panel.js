// Build a model in the browser and run it over the source.
//
// The selects only ever assemble a DSL string, which is shown and editable. Rust does the
// parsing, so the panel never learns what a prefix model or a counter actually is.

import { on } from '../core/bus.js';
import { commas, fixed3 } from '../core/helpers.js';
import { model } from '../core/model.js';
import { source } from '../core/source.js';

const FAMILIES = [
  ['PM', 'unaligned'],
  ['BytePM', 'byte aligned'],
  ['NibblePM', 'nibble aligned'],
  ['BitPM', 'bit aligned'],
];
const WIDTHS = [4, 7, 8, 12, 16];
const COUNTERS = ['Counter4'];
const HISTORIES = ['Raw', 'AC', 'Huff'];

const opts = (values, labels) => values
  .map((v, i) => `<option value="${v}">${labels ? labels[i] : v}</option>`)
  .join('');

export class ModelPanel extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `
      <h2>model</h2>
      <label>prefix model <select class="family">
        ${opts(FAMILIES.map((f) => f[0]), FAMILIES.map((f) => f[1]))}</select></label>
      <label>context bits <select class="width">${opts(WIDTHS)}</select></label>
      <label>counter <select class="counter">${opts(COUNTERS)}</select></label>

      <h2>history</h2>
      <label>kind <select class="history">${opts(HISTORIES)}</select></label>
      <div class="ac-args" hidden>
        <label>ac bits <input class="ac-bits" type="number" value="12" min="1" max="32"></label>
        <label>inner model <select class="ac-family">
          ${opts(FAMILIES.map((f) => f[0]), FAMILIES.map((f) => f[1]))}</select></label>
        <label>inner bits <select class="ac-width">${opts(WIDTHS)}</select></label>
      </div>
      <div class="huff-args" hidden>
        <label>huff size <input class="huff" type="number" value="11" min="1" max="16"></label>
        <label>rem size <input class="rem" type="number" value="11" min="1" max="16"></label>
      </div>

      <h2>spec</h2>
      <textarea class="spec" rows="3" spellcheck="false"></textarea>
      <div class="row-buttons">
        <button class="run">Run</button>
        <button class="sync">Reset to selects</button>
      </div>
      <p class="sub status"></p>`;

    this.spec = this.querySelector('.spec');
    this.status = this.querySelector('.status');

    this.ac = new AbortController();
    const { signal } = this.ac;

    for (const sel of this.querySelectorAll('select, input')) {
      sel.addEventListener('change', () => this.sync(), { signal });
    }
    this.querySelector('.run').addEventListener('click', () => this.run(), { signal });
    this.querySelector('.sync').addEventListener('click', () => this.sync(), { signal });
    on('model:done', () => this.report(), signal);

    this.sync();
  }

  disconnectedCallback() {
    this.ac.abort();
  }

  /** The two strings rust parses. `spec` holds them one per line, so it can be edited. */
  specs() {
    const [model, history] = this.spec.value.split('\n').map((s) => s.trim());
    return [model, history || 'Raw'];
  }

  sync() {
    const pick = (c) => this.querySelector(c).value;
    const kind = pick('.history');
    this.querySelector('.ac-args').hidden = kind !== 'AC';
    this.querySelector('.huff-args').hidden = kind !== 'Huff';

    const model = `${pick('.family')}${pick('.width')}(${pick('.counter')})`;
    const history = kind === 'AC'
      ? `AC(${pick('.ac-bits')}, ${pick('.ac-family')}${pick('.ac-width')}(${pick('.counter')}))`
      : kind === 'Huff' ? `Huff(${pick('.huff')}, ${pick('.rem')})` : 'Raw';

    this.spec.value = `${model}\n${history}`;
  }

  async run() {
    if (!source.layout.done) return this.note('load a source file first');
    const [modelSpec, historySpec] = this.specs();
    this.note(`running ${modelSpec} over ${historySpec}...`);
    try {
      await model.build(modelSpec, historySpec);
    } catch (err) {
      this.note(String(err.message || err));
    }
  }

  report() {
    this.note(`${commas(Math.round(model.entropy / 8))} B, ratio ${fixed3(model.cr)}`);
  }

  note(msg) {
    this.status.textContent = msg;
  }
}

customElements.define('x-model-panel', ModelPanel);
