// The file pickers and what the loaded files add up to. One row per whole file today, one
// per component once the model reports them.

import { commas, fixed3 } from '../analyzer.js';
import { on } from '../core/bus.js';
import { model } from '../core/model.js';
import { source } from '../core/source.js';

export class SourcePanel extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `
      <h2>files</h2>
      <div id="files-slot"></div>
      <table hidden>
        <thead><tr><th></th><th>size</th><th>&Sigma; entropy</th><th>ratio</th></tr></thead>
        <tbody><tr><th>total</th><td class="size">-</td><td class="entropy">-</td><td class="ratio">-</td></tr></tbody>
      </table>`;

    this.table = this.querySelector('table');
    this.ac = new AbortController();
    const { signal } = this.ac;
    for (const name of ['source:grow', 'source:done', 'model:grow', 'model:done']) {
      on(name, () => this.render(), signal);
    }
    this.render();
  }

  disconnectedCallback() {
    this.ac.abort();
  }

  render() {
    this.table.hidden = !source.size;
    if (this.table.hidden) return;
    this.querySelector('.size').textContent = commas(source.size) + ' B';
    this.querySelector('.entropy').textContent = model.nProbs
      ? commas(Math.round(model.entropy / 8)) + ' B' : '-';
    this.querySelector('.ratio').textContent = model.nProbs ? fixed3(model.cr) : '-';
  }
}

customElements.define('x-source-panel', SourcePanel);
