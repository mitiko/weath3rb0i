// The 8 bits of the anchored byte: what was coded, what the model thought, what it cost.

import { bitAt, costBits, prob } from '../analyzer.js';
import { bitBucket } from '../color.js';
import { on } from '../core/bus.js';
import { escape, fixed3, hex, pct } from '../core/helpers.js';
import { model } from '../core/model.js';
import { source } from '../core/source.js';
import { setAnchor, view } from '../core/view.js';
import { glyph } from '../layout.js';

export class BitStrip extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `
      <h2>bits</h2>
      <p class="hint">Click a character to anchor a position.</p>
      <div id="bitstrip" hidden></div>
      <table class="kv" hidden></table>`;
    this.hint = this.querySelector('.hint');
    this.strip = this.querySelector('#bitstrip');
    this.summary = this.querySelector('.kv');

    this.ac = new AbortController();
    const { signal } = this.ac;
    on('anchor', () => this.render(), signal);
    on('model:grow', () => this.render(), signal);
    on('cr', () => this.render(), signal);
    this.strip.addEventListener('click', (e) => {
      const bit = e.target.closest('.bit');
      if (bit) setAnchor(+bit.dataset.pos);
    }, { signal });

    this.render();
  }

  disconnectedCallback() {
    this.ac.abort();
  }

  render() {
    const idle = view.anchor === null;
    this.hint.hidden = !idle;
    this.strip.hidden = idle;
    this.summary.hidden = idle;
    if (idle) return;

    const byte = view.anchor >> 3;
    const base = byte << 3;
    let html = '';
    let total = 0;
    let known = true;

    for (let k = 0; k < 8; k++) {
      const at = base + k;
      const bit = bitAt(source.bytes, at);
      if (at >= model.nProbs) {
        known = false;
        html += `<div class="bit eu" data-pos="${at}"><div class="v">${bit}</div>` +
                '<div class="u">-</div><div class="p">-</div><div class="c">-</div></div>';
        continue;
      }
      const p = model.probs[at];
      const cost = costBits(p, bit);
      total += cost;
      const here = at === view.anchor ? ' on' : '';
      html += `<div class="bit e${bitBucket(cost)}${here}" data-pos="${at}" ` +
              `title="bit ${at}: p ${p}/65536 = ${prob(p)}, costs ${cost} bits">` +
              `<div class="v">${bit}</div>` +
              `<div class="u">${p}</div>` +
              `<div class="p">${fixed3(prob(p))}</div>` +
              `<div class="c">${fixed3(cost)}</div></div>`;
    }
    this.strip.innerHTML = html;

    const b = source.bytes[byte];
    // P(byte) is what the 8 predictions jointly assigned to this character. Three decimals
    // of a percent: anything smaller reads as 0.000% rather than as an exponent.
    const joint = known ? pct(2 ** -total) : '-';
    this.summary.innerHTML = rows([
      ['character', `'${escape(glyph(b).text)}'`],
      ['hex code', '0x' + hex(b)],
      ['bpc', known ? fixed3(total) : '-'],
      ['P(byte)', joint],
    ]);
  }
}

const rows = (pairs) => pairs.map(([k, v]) => `<tr><th>${k}</th><td>${v}</td></tr>`).join('');

customElements.define('x-bit-strip', BitStrip);
