// The colour ramp, and the band filter you drag across it.
//
// The filter narrows what the whole page colours, which is how you look at just the
// predictable or just the random characters. It only sets view.filter: color.js rewrites
// the 32 rules and every element using them follows without re-rendering.

import { BUCKETS, bucketColor, costFor } from '../color.js';
import { on } from '../core/bus.js';
import { setFilter, view } from '../core/view.js';

export class HeatMap extends HTMLElement {
  connectedCallback() {
    this.ramp = document.createElement('span');
    this.ramp.className = 'ramp';
    for (let i = 0; i < BUCKETS; i++) {
      const seg = document.createElement('span');
      seg.style.background = bucketColor(i);
      seg.dataset.b = i;
      this.ramp.append(seg);
    }
    this.replaceChildren(this.label('predictable'), this.ramp, this.label('random'));

    this.ac = new AbortController();
    const { signal } = this.ac;
    on('filter', () => this.mark(), signal);

    this.ramp.addEventListener('mouseover', (e) => {
      const seg = e.target.closest('span[data-b]');
      if (seg) this.showTip(seg, +seg.dataset.b);
    }, { signal });
    this.ramp.addEventListener('mouseleave', () => this.hideTip(), { signal });

    // drag on the document, so a drag that runs past either end still clamps and tracks
    let from = null;
    this.ramp.addEventListener('mousedown', (e) => {
      e.preventDefault();
      from = this.bucketAtX(e.clientX);
      setFilter([from, from]);
    }, { signal });
    addEventListener('mousemove', (e) => {
      if (from === null) return;
      const i = this.bucketAtX(e.clientX);
      setFilter([Math.min(from, i), Math.max(from, i)]);
    }, { signal });
    addEventListener('mouseup', () => { from = null; }, { signal });
  }

  disconnectedCallback() {
    this.ac.abort();
    this.tip?.remove();
  }

  label(name) {
    const el = document.createElement('span');
    el.className = 'lab';
    el.textContent = name;
    el.title = 'show the whole range again';
    el.onclick = () => setFilter(null);
    return el;
  }

  /** Dim the segments outside the band, so the ramp shows what is selected. */
  mark() {
    const [lo, hi] = view.filter || [0, BUCKETS - 1];
    for (const seg of this.ramp.children) {
      seg.classList.toggle('off', +seg.dataset.b < lo || +seg.dataset.b > hi);
    }
  }

  // read from geometry rather than the event target, so a drag keeps tracking off the ends
  bucketAtX(x) {
    const r = this.ramp.getBoundingClientRect();
    return Math.min(BUCKETS - 1, Math.max(0, Math.floor(((x - r.left) / r.width) * BUCKETS)));
  }

  // The band of bits per bit a bucket covers, and the same band against the baseline.
  // bucket() rounds to nearest, so a bucket reaches half a step either side of its colour.
  tipHtml(i) {
    const step = 1 / (BUCKETS - 1);
    const lo = costFor(Math.max(0, (i - 0.5) * step)) / 8;
    const hi = costFor(Math.min(1, (i + 0.5) * step)) / 8;
    const top = i === BUCKETS - 1;
    return `<b>${lo.toFixed(2)}${top ? '+' : ` - ${hi.toFixed(2)}`}</b> bits per bit`
      + `<br>${(lo / view.cr).toFixed(2)}${top ? '×+' : ` - ${(hi / view.cr).toFixed(2)}×`} baseline`;
  }

  showTip(seg, i) {
    if (!this.tip) {
      this.tip = document.createElement('div');
      this.tip.id = 'ramp-tip';
      document.body.append(this.tip);
    }
    this.tip.innerHTML = this.tipHtml(i);
    this.tip.hidden = false;
    const r = seg.getBoundingClientRect();
    this.tip.style.left = `${r.left + r.width / 2}px`;
    this.tip.style.top = `${r.bottom + 8}px`;
  }

  hideTip() {
    if (this.tip) this.tip.hidden = true;
  }
}

customElements.define('x-heat-map', HeatMap);
