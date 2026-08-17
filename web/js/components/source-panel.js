// The file pickers and what the loaded files add up to.

import { commas, fixed3 } from '../analyzer.js';
import { on } from '../core/bus.js';
import { model } from '../core/model.js';
import { source } from '../core/source.js';
import { rank } from '../ltcb.js';

export class SourcePanel extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `
      <h2>files</h2>
      <div id="files-slot"></div>
      <table hidden>
        <tbody>
          <tr><th>size</th><td class="size">-</td></tr>
          <tr><th>entropy sum</th><td class="entropy">-</td></tr>
          <tr><th>ratio</th><td class="ratio">-</td></tr>
          <tr><th>LTCB position</th><td class="ltcb">-</td></tr>
        </tbody>
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

    const cell = (sel) => this.querySelector(sel);
    cell('.size').textContent = commas(source.size) + ' B';
    cell('.entropy').textContent = model.nProbs
      ? commas(Math.round(model.entropy / 8)) + ' B' : '-';
    cell('.ratio').textContent = model.nProbs ? fixed3(model.cr) : '-';

    const ltcb = cell('.ltcb');
    if (!model.nProbs) {
      ltcb.textContent = '-';
      ltcb.title = '';
      return;
    }
    const { position, total, near } = rank(model.cr);
    ltcb.textContent = `~${position} / ${total}`;
    ltcb.title = `about where ${near} lands on enwik9. Different corpus, so this places the `
      + 'ratio against the benchmark, it is not a score on it';
  }
}

customElements.define('x-source-panel', SourcePanel);
