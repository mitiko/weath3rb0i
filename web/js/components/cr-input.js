// The colour scale midpoint, in bits per bit. It shows the ratio measured off the files;
// type a value to pin it, click the CR label to go back to the measured one.

import { fixed3 } from '../analyzer.js';
import { on } from '../core/bus.js';
import { setCR, view } from '../core/view.js';

export class CrInput extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `
      <input type="number" step="0.001" min="0.001" max="4"
             title="colour scale midpoint, in bits per bit. Type a value to pin it">
      <i title="reset to the ratio measured from the files">CR</i>`;

    this.input = this.querySelector('input');
    this.ac = new AbortController();
    const { signal } = this.ac;

    on('cr', () => this.render(), signal);
    this.input.addEventListener('change', () => {
      const typed = +this.input.value;
      if (typed > 0) setCR(typed, true);
      else this.reset();
    }, { signal });
    this.querySelector('i').addEventListener('click', () => this.reset(), { signal });

    this.render();
  }

  disconnectedCallback() {
    this.ac.abort();
  }

  reset() {
    if (view.measured) setCR(view.measured);
  }

  render() {
    this.input.value = view.measured || view.pinned ? fixed3(view.cr) : '';
  }
}

customElements.define('x-cr-input', CrInput);
