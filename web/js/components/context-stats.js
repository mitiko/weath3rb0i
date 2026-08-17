// How often this character appears in its context, and what tends to follow it. Source
// only: no model output is involved, so this works before any probabilities are loaded.

import { commas } from '../analyzer.js';
import { on } from '../core/bus.js';
import { source } from '../core/source.js';
import { view } from '../core/view.js';
import { glyph } from '../layout.js';

const DEPTH = 3; // context depths beyond the character itself

const ESC_HTML = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' };
const escape = (s) => String(s).replace(/[&<>"]/g, (c) => ESC_HTML[c]);

export class ContextStats extends HTMLElement {
  connectedCallback() {
    this.innerHTML = '<h2>context</h2><div id="ctxstats"></div>';
    this.out = this.querySelector('#ctxstats');

    this.ac = new AbortController();
    // counting scans the whole loaded source, so only redo it when the anchor moves
    on('anchor', () => this.render(), this.ac.signal);
    this.render();
  }

  disconnectedCallback() {
    this.ac.abort();
  }

  render() {
    this.hidden = view.anchor === null;
    if (this.hidden) return;

    const i = view.anchor >> 3;
    // only search the part of the source that has actually streamed in
    const n = source.layout.done;
    const bytes = source.bytes.subarray(0, n);
    const here = [];  // counts of X, aX, baX, cbaX
    const next = [];  // counts of XY, aXY, baXY, cbaXY

    for (let d = 0; d <= DEPTH; d++) {
      if (i - d < 0) break;
      here.push({ pattern: bytes.subarray(i - d, i + 1), count: count(bytes, bytes.subarray(i - d, i + 1)) });
      if (i + 1 < n) {
        next.push({ pattern: bytes.subarray(i - d, i + 2), count: count(bytes, bytes.subarray(i - d, i + 2)) });
      }
    }

    let html = table('this character', ['context', 'count', 'share'], here.map((r, k) => {
      const denom = k === 0 ? n : here[k - 1].count;
      const label = k === 0 ? 'of file' : `of ${show(here[k - 1].pattern)}`;
      return [show(r.pattern), commas(r.count), `${pct(r.count, denom)} ${label}`];
    }));

    html += table('next character', ['context', 'count', 'P(next)'], next.map((r, k) => [
      show(r.pattern), commas(r.count), `${pct(r.count, here[k].count)} of ${show(here[k].pattern)}`,
    ]));

    this.out.innerHTML = html;
  }
}

/** Occurrences of `needle` in `bytes`. */
function count(bytes, needle) {
  const n = bytes.length, m = needle.length;
  const first = needle[0];
  let hits = 0;
  outer:
  for (let i = 0; i + m <= n; i++) {
    if (bytes[i] !== first) continue;
    for (let j = 1; j < m; j++) if (bytes[i + j] !== needle[j]) continue outer;
    hits++;
  }
  return hits;
}

const show = (pat) => escape(Array.from(pat, (b) => glyph(b).text).join(''));
const pct = (a, b) => (b > 0 ? (100 * a / b).toFixed(3) + '%' : '-');

function table(caption, head, rows) {
  let html = `<table><caption>${caption}</caption><tr>`;
  for (const h of head) html += `<th>${h}</th>`;
  html += '</tr>';
  for (const r of rows) {
    html += '<tr>';
    for (const c of r) html += `<td>${c}</td>`;
    html += '</tr>';
  }
  return html + '</table>';
}

customElements.define('x-context-stats', ContextStats);
