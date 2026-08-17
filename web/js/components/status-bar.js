// Source name and the position readout. Hovering a character previews its position; a
// click holds it, and the stat jumps back to the held one.

import { emit, on } from '../core/bus.js';
import { commas } from '../core/helpers.js';
import { source } from '../core/source.js';
import { view } from '../core/view.js';

export class StatusBar extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `
      <span class="stat"><b id="s-source">-</b><i>source</i></span>
      <span class="stat jump" title="jump back to the anchored character">
        <b id="s-pos">-</b><i>pos</i></span>`;

    this.name = this.querySelector('#s-source');
    this.pos = this.querySelector('#s-pos');

    this.ac = new AbortController();
    const { signal } = this.ac;
    on('source:grow', () => this.renderName(), signal);
    on('source:done', () => this.renderName(), signal);
    on('anchor', () => this.renderPos(), signal);
    on('hover', () => this.renderPos(), signal);
    this.querySelector('.jump').addEventListener('click', () => emit('jump'), { signal });
  }

  disconnectedCallback() {
    this.ac.abort();
  }

  renderName() {
    this.name.textContent = source.name || '-';
    this.name.title = source.sha256 ? 'sha256 ' + source.sha256 : '';
  }

  renderPos() {
    const pos = view.hover === null ? view.anchor : view.hover;
    this.pos.textContent = pos === null ? '-' : commas(pos);
  }
}

customElements.define('x-status-bar', StatusBar);
