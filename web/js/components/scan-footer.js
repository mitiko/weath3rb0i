// Load progress for the three files, plus tab health.
//
// There is no CPU API. The figure is how late a fixed-interval timer fires, which tracks
// main-thread blocking and is most useful while scrolling. Memory has no portable API at
// all, so that counter reads N/A.

import { commas } from '../analyzer.js';
import { on } from '../core/bus.js';
import { model } from '../core/model.js';
import { source } from '../core/source.js';

const CPU_TICK = 500;

export class ScanFooter extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `
      <span class="prog">source <progress id="src-progress" value="0" max="1"></progress></span>
      <span class="prog">probs <progress id="probs-progress" value="0" max="1"></progress></span>
      <span class="prog">index <progress id="index-progress" value="0" max="1"></progress></span>
      <span id="scan-status"></span>
      <span class="meter" title="main-thread blocking, estimated from timer drift">cpu <b id="m-cpu">-</b></span>
      <span class="meter" title="no portable way to read it: performance.memory is Chrome-only and measureUserAgentSpecificMemory() needs COOP/COEP headers">mem <b>N/A</b></span>`;

    this.src = this.querySelector('#src-progress');
    this.probs = this.querySelector('#probs-progress');
    this.index = this.querySelector('#index-progress');
    this.status = this.querySelector('#scan-status');
    this.cpu = this.querySelector('#m-cpu');

    this.ac = new AbortController();
    for (const name of ['source:grow', 'source:done', 'model:grow', 'model:done', 'machine:grow', 'machine:done']) {
      on(name, () => this.render(), this.ac.signal);
    }

    let last = performance.now();
    this.timer = setInterval(() => {
      const now = performance.now();
      const elapsed = now - last;
      last = now;
      const busy = Math.min(1, Math.max(0, (elapsed - CPU_TICK) / elapsed));
      this.cpu.textContent = Math.round(busy * 100) + '%';
    }, CPU_TICK);

    this.render();
  }

  disconnectedCallback() {
    this.ac.abort();
    clearInterval(this.timer);
  }

  render() {
    const machine = model.state;
    this.src.value = source.size ? source.loaded / source.size : 0;
    this.probs.value = model.size ? (model.loaded * 2) / model.size : 0;
    this.index.value = machine && machine.total ? machine.loaded / machine.total : 0;
    const msg = this.warning();
    this.status.textContent = msg ? '! ' + msg : '';
  }

  // derived from what is loaded rather than latched at load time, so it is never stale
  warning() {
    const machine = model.state;
    if (machine && machine.error) return 'index failed: ' + machine.error;
    if (model.size && source.size && model.size !== source.size * 16) {
      return `${commas(model.size / 2)} probabilities for ${commas(source.size * 8)} bits, the files may not match`;
    }
    if (machine && machine.complete && source.complete && machine.lines !== source.bytes.length * 8) {
      return `state file has ${commas(machine.lines)} lines for ${commas(source.bytes.length * 8)} bits`;
    }
    return '';
  }
}

customElements.define('x-scan-footer', ScanFooter);
