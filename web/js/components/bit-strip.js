// The 8 bits of the anchored byte: what was coded, what the model thought, what it cost.

import { bitAt, costBits, fixed3, prob } from '../analyzer.js';
import { bitBucket } from '../color.js';
import { on } from '../core/bus.js';
import { model } from '../core/model.js';
import { source } from '../core/source.js';
import { setAnchor, view } from '../core/view.js';
import { glyph } from '../layout.js';

export class BitStrip extends HTMLElement {
  connectedCallback() {
    this.innerHTML = '<h2>bits</h2><div id="bitstrip"></div><p class="sub"></p>';
    this.strip = this.querySelector('#bitstrip');
    this.summary = this.querySelector('.sub');

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
    this.hidden = view.anchor === null;
    if (this.hidden) return;

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
      const on_ = at === view.anchor ? ' on' : '';
      html += `<div class="bit e${bitBucket(cost)}${on_}" data-pos="${at}" ` +
              `title="bit ${at}: p ${p}/65536 = ${prob(p)}, costs ${cost} bits">` +
              `<div class="v">${bit}</div>` +
              `<div class="u">${p}</div>` +
              `<div class="p">${fixed3(prob(p))}</div>` +
              `<div class="c">${fixed3(cost)}</div></div>`;
    }
    this.strip.innerHTML = html;

    const g = glyph(source.bytes[byte]);
    const hex = source.bytes[byte].toString(16).padStart(2, '0');
    // P(byte) is what the 8 predictions jointly assigned to this character
    const joint = 2 ** -total;
    this.summary.textContent = known
      ? `'${g.text}' 0x${hex} - ${fixed3(total)} bpc - P(byte) ` +
        (joint >= 0.001 ? fixed3(joint) : joint.toExponential(2))
      : `'${g.text}' 0x${hex} - not fully loaded`;
  }
}

customElements.define('x-bit-strip', BitStrip);
