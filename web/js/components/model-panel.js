// Build a model in the browser and run it over the source.
//
// The selects only ever assemble a DSL string, which is shown and editable. Rust does the
// parsing, so the panel never learns what a prefix model or a counter actually is.

import { on } from '../core/bus.js';
import { commas, fixed3 } from '../core/helpers.js';
import { model } from '../core/model.js';
import { source } from '../core/source.js';
import { view } from '../core/view.js';

const MODELS = [
  ['PM5', 'order-0 nibble tree, 5 bits'],
  ['PM8', 'order-0 byte tree, 8 bits'],
  ['PM9', 'order-0 nibble tree, 9 bits'],
  ['PM13', 'order-2 nibble tree, 13 bits'],
  ['PM16', 'order-1 byte tree, 16 bits'],
];
const COUNTERS = ['Counter4'];
const HISTORIES = ['Raw', 'AC', 'Huff'];

const opts = (values, labels) => values
  .map((v, i) => `<option value="${v}">${labels ? labels[i] : v}</option>`)
  .join('');

export class ModelPanel extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `
      <h2>model</h2>
      <label>prefix model <select class="model">
        ${opts(MODELS.map((m) => m[0]), MODELS.map((m) => m[1]))}</select></label>
      <label>counter <select class="counter">${opts(COUNTERS)}</select></label>

      <h2>history</h2>
      <label>kind <select class="history">${opts(HISTORIES)}</select></label>
      <div class="ac-args" hidden>
        <label>ac bits <input class="ac-bits" type="number" value="12" min="1" max="32"></label>
        <label>inner model <select class="ac-model">
          ${opts(MODELS.map((m) => m[0]), MODELS.map((m) => m[1]))}</select></label>
      </div>
      <div class="huff-args" hidden>
        <label>huff size <input class="huff" type="number" value="11" min="1" max="16"></label>
        <label>rem size <input class="rem" type="number" value="11" min="1" max="16"></label>
      </div>

      <h2>spec</h2>
      <textarea class="spec" rows="3" spellcheck="false"></textarea>
      <div class="row-buttons">
        <button class="run" title="encode up to the anchored position">Run</button>
        <button class="run-all" title="encode the whole file">Run all</button>
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
    this.querySelector('.run').addEventListener('click', () => this.run(false), { signal });
    this.querySelector('.run-all').addEventListener('click', () => this.run(true), { signal });
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

    const model = `${pick('.model')}(${pick('.counter')})`;
    const history = kind === 'AC'
      ? `AC(${pick('.ac-bits')}, ${pick('.ac-model')}(${pick('.counter')}))`
      : kind === 'Huff' ? `Huff(${pick('.huff')}, ${pick('.rem')})` : 'Raw';

    this.spec.value = `${model}\n${history}`;
  }

  // `all` runs the whole file. Otherwise it runs to the anchor, continuing from where the
  // model stands, or replaying from the start when the anchor is behind it.
  async run(all) {
    if (!source.layout.done) return this.note('load a source file first');
    if (!all && view.anchor === null) {
      alert('Click a character in the text to pick a position to run to.');
      return this.note('no position selected');
    }

    const [modelSpec, historySpec] = this.specs();
    const target = all ? source.layout.done * 8 : view.anchor;
    try {
      if (model.name !== `${modelSpec} over ${historySpec}`) {
        await model.build(modelSpec, historySpec);
      }
      const behind = target < model.state.position;
      this.note(behind ? 'replaying from the start...' : 'running...');
      await model.runTo(target);
      this.report();
    } catch (err) {
      this.note(String(err.message || err));
    }
  }

  report() {
    const at = model.state ? model.state.position : 0;
    const kept = model.state ? model.state.taken.size : 0;
    this.note(`${commas(Math.round(model.entropy / 8))} B, ratio ${fixed3(model.cr)}`
      + ` - at bit ${commas(at)}, ${kept} checkpoint${kept === 1 ? '' : 's'}`);
  }

  note(msg) {
    this.status.textContent = msg;
  }
}

customElements.define('x-model-panel', ModelPanel);
