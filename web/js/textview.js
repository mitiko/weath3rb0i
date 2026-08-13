// The virtualized 80-column text grid. The window scrolls, not an inner box, so the
// offset gutter can hang into the left margin without being clipped.

import { glyph } from './layout.js';
import { charCost } from './analyzer.js';
import { bucket } from './color.js';

const ROW_H = 24;
const OVERSCAN = 8;

const ESC_HTML = { '&': '&amp;', '<': '&lt;', '>': '&gt;' };
const escape = (s) => s.replace(/[&<>]/g, (c) => ESC_HTML[c]);

export class TextView {
  constructor(state, { onHover, onPick }) {
    this.state = state;
    this.onHover = onHover;
    this.onPick = onPick;
    this.main = document.getElementById('main');
    this.sizer = document.getElementById('sizer');
    this.rows = document.getElementById('rows');
    this.first = -1;
    this.last = -1;
    this.pending = false;

    document.documentElement.style.setProperty('--row-h', ROW_H + 'px');

    addEventListener('scroll', () => this.schedule(), { passive: true });
    addEventListener('resize', () => this.schedule(), { passive: true });

    this.rows.addEventListener('mouseover', (e) => {
      const i = e.target.dataset && e.target.dataset.i;
      if (i !== undefined) this.onHover(+i);
    });
    this.rows.addEventListener('mouseleave', () => this.onHover(null));
    this.rows.addEventListener('click', (e) => {
      const i = e.target.dataset && e.target.dataset.i;
      if (i !== undefined) this.onPick(+i);
    });
  }

  /** Re-render on the next frame. */
  schedule() {
    if (this.pending) return;
    this.pending = true;
    requestAnimationFrame(() => {
      this.pending = false;
      this.render();
    });
  }

  /** Discard the cached window so the next render rebuilds it. */
  invalidate() {
    this.first = this.last = -1;
    this.schedule();
  }

  render() {
    const { layout } = this.state;
    if (!layout) return;

    const total = layout.rows;
    this.sizer.style.height = total * ROW_H + 'px';

    const top = Math.max(0, scrollY - this.main.offsetTop);
    const first = Math.max(0, Math.floor(top / ROW_H) - OVERSCAN);
    const last = Math.min(total, Math.ceil((top + innerHeight) / ROW_H) + OVERSCAN);
    if (first === this.first && last === this.last) return;
    this.first = first;
    this.last = last;

    let html = '';
    for (let row = first; row < last; row++) html += this.rowHtml(row);
    this.rows.style.top = first * ROW_H + 'px';
    this.rows.innerHTML = html;
    this.markAnchor();
  }

  rowHtml(row) {
    const { layout, bytes, probs, nProbs } = this.state;
    const [from, to] = layout.range(row);
    let out = `<div class="trow"><span class="gutter">${from}</span>`;
    for (let i = from; i < to; i++) {
      const g = glyph(bytes[i]);
      const cost = charCost(probs, nProbs, bytes, i);
      const cls = cost < 0 ? 'eu' : 'e' + bucket(cost);
      out += `<span class="${cls}" data-i="${i}">${escape(g.text)}</span>`;
    }
    return out + '</div>';
  }

  /** Outline the anchored byte, if it is on screen. */
  markAnchor() {
    const prev = this.rows.querySelector('.anchor');
    if (prev) prev.classList.remove('anchor');
    const anchor = this.state.anchor;
    if (anchor === null) return;
    const el = this.rows.querySelector(`span[data-i="${anchor >> 3}"]`);
    if (el) el.classList.add('anchor');
  }
}
