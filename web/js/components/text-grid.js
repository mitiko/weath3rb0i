// The virtualized 80-column text grid.
//
// This element is the sizer: its height is the whole file, so the native scrollbar is
// honest, while only the visible rows plus overscan exist in the DOM. The window scrolls,
// not an inner box, so the offset gutter can hang into the left margin without clipping.

import { charCost } from '../analyzer.js';
import { bucket } from '../color.js';
import { on } from '../core/bus.js';
import { model } from '../core/model.js';
import { hitAt, search } from '../core/search.js';
import { source } from '../core/source.js';
import { setAnchor, setHover, view } from '../core/view.js';
import { glyph } from '../layout.js';

const ROW_H = 24;
const OVERSCAN = 8;

const ESC_HTML = { '&': '&amp;', '<': '&lt;', '>': '&gt;' };
const escape = (s) => s.replace(/[&<>]/g, (c) => ESC_HTML[c]);

export class TextGrid extends HTMLElement {
  connectedCallback() {
    this.rows = document.createElement('div');
    this.rows.id = 'rows';
    this.replaceChildren(this.rows);
    this.first = -1;
    this.last = -1;
    this.dirty = true;
    this.pending = false;

    document.documentElement.style.setProperty('--row-h', ROW_H + 'px');

    this.ac = new AbortController();
    const { signal } = this.ac;

    addEventListener('scroll', () => this.schedule(), { passive: true, signal });
    addEventListener('resize', () => this.schedule(), { passive: true, signal });

    // more text, more probabilities, or a new cost-to-bucket mapping all change the classes
    on('source:grow', () => this.repaint(), signal);
    on('model:grow', () => this.repaint(), signal);
    on('cr', () => this.repaint(), signal);
    // a filter change is 32 rewritten CSS rules and nothing else, so there is nothing to do
    on('anchor', () => this.markAnchor(), signal);
    on('jump', () => this.scrollTo(view.anchor >> 3), signal);
    on('search', () => this.repaint(), signal);
    on('search:goto', () => this.scrollTo(search.matches[search.at]), signal);

    this.rows.addEventListener('mouseover', (e) => {
      const i = e.target.dataset && e.target.dataset.i;
      if (i !== undefined) setHover(+i * 8);
    }, { signal });
    this.rows.addEventListener('mouseleave', () => setHover(null), { signal });
    this.rows.addEventListener('click', (e) => {
      // a drag-select ends in a click, and moving the anchor then is never what was meant
      if (!getSelection().isCollapsed) return;
      const i = e.target.dataset && e.target.dataset.i;
      // every listener speaks in bit positions; a character click lands on its first bit
      if (i !== undefined) setAnchor(+i * 8);
    }, { signal });

    this.schedule();
  }

  disconnectedCallback() {
    this.ac.abort();
  }

  /** Render on the next frame. */
  schedule() {
    if (this.pending) return;
    this.pending = true;
    requestAnimationFrame(() => {
      this.pending = false;
      this.render();
    });
  }

  /** The content changed, so the visible rows have to be rebuilt even if they are the same rows. */
  repaint() {
    this.dirty = true;
    this.schedule();
  }

  /** Bring the row holding `byte` a third of the way down the window. */
  scrollTo(byte) {
    if (byte === undefined || byte === null) return;
    const row = source.layout.rowOf(byte);
    scrollTo({ top: this.offsetTop + row * ROW_H - innerHeight / 3, behavior: 'smooth' });
  }

  render() {
    const { layout } = source;
    const total = layout.rows;
    this.style.height = total * ROW_H + 'px';

    const top = Math.max(0, scrollY - this.offsetTop);
    const first = Math.max(0, Math.floor(top / ROW_H) - OVERSCAN);
    const last = Math.min(total, Math.ceil((top + innerHeight) / ROW_H) + OVERSCAN);
    // scroll fires constantly but the row window only moves every ROW_H pixels
    if (!this.dirty && first === this.first && last === this.last) return;
    this.dirty = false;
    this.first = first;
    this.last = last;

    let html = '';
    for (let row = first; row < last; row++) html += this.rowHtml(row);
    this.rows.style.top = first * ROW_H + 'px';
    this.rows.innerHTML = html;
    this.markAnchor();
  }

  rowHtml(row) {
    const [from, to] = source.layout.range(row);
    let out = `<div class="trow"><span class="gutter">${from}</span>`;
    for (let i = from; i < to; i++) {
      const g = glyph(source.bytes[i]);
      const cost = charCost(model.probs, model.nProbs, source.bytes, i);
      const hit = hitAt(i);
      let cls = cost < 0 ? 'eu' : 'e' + bucket(cost);
      if (hit) cls += hit === 2 ? ' hit now' : ' hit';
      out += `<span class="${cls}" data-i="${i}">${escape(g.text)}</span>`;
    }
    return out + '</div>';
  }

  /** Outline the anchored byte and flag its row, so it stays findable while scrolling. */
  markAnchor() {
    for (const prev of this.rows.querySelectorAll('.anchor, .anchor-row')) {
      prev.classList.remove('anchor', 'anchor-row');
    }
    if (view.anchor === null) return;
    const el = this.rows.querySelector(`span[data-i="${view.anchor >> 3}"]`);
    if (!el) return;
    el.classList.add('anchor');
    el.parentElement.classList.add('anchor-row');
  }
}

customElements.define('x-text-grid', TextGrid);
