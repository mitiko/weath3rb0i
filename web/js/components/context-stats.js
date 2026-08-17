// How predictable this character is from what precedes it, and what tends to follow it.
// Source only: no model output involved, so it works before any probabilities are loaded.
//
// Both tables condition forwards, P(char | context). The reverse reading, "of all 'd',
// this share was preceded by 'e'", is a fact about the corpus but not about how hard the
// character was to predict, which is the question the rest of the page is asking.

import { commas } from '../analyzer.js';
import { on } from '../core/bus.js';
import { source } from '../core/source.js';
import { view } from '../core/view.js';
import { glyph } from '../layout.js';

const DEPTH = 5; // context characters beyond the one anchored

const ESC_HTML = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' };
const escape = (s) => String(s).replace(/[&<>"]/g, (c) => ESC_HTML[c]);

export class ContextStats extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `
      <h2>context</h2>
      <p class="hint">Click a character to anchor a position.</p>
      <div id="ctxstats"></div>`;
    this.hint = this.querySelector('.hint');
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
    const idle = view.anchor === null;
    this.hint.hidden = !idle;
    this.out.hidden = idle;
    if (idle) return;

    const i = view.anchor >> 3;
    // only search the part of the source that has actually streamed in
    const n = source.layout.done;
    const bytes = source.bytes.subarray(0, n);
    const here = [];
    const next = [];

    for (let d = 0; d <= DEPTH; d++) {
      if (i - d < 0) break;
      const ctx = bytes.subarray(i - d, i);          // what precedes the character
      const seq = bytes.subarray(i - d, i + 1);      // context plus the character
      here.push({ seq, ctx, count: count(bytes, seq), of: count(bytes, ctx) });
      if (i + 1 < n) {
        const grown = bytes.subarray(i - d, i + 2);  // and plus the one after it
        next.push({ seq: grown, ctx: seq, count: count(bytes, grown), of: here[d].count });
      }
    }

    const char = quote(bytes[i]);
    const after = i + 1 < n ? quote(bytes[i + 1]) : '';

    let html = table('this character', ['sequence', 'count', 'probability'], here.map((r) => [
      show(r.seq), commas(r.count), `${cond(char, r.ctx)} ${pct(r.count, r.of)}`,
    ]));

    html += table('next character', ['sequence', 'count', 'probability'], next.map((r) => [
      show(r.seq), commas(r.count), `${cond(after, r.ctx)} ${pct(r.count, r.of)}`,
    ]));

    this.out.innerHTML = html;
  }
}

/** Occurrences of `needle` in `bytes`. */
function count(bytes, needle) {
  const n = bytes.length, m = needle.length;
  if (m === 0) return n;
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

const text = (pat) => escape(Array.from(pat, (b) => glyph(b).text).join(''));
const show = (pat) => `'${text(pat)}'`;
const quote = (b) => `'${escape(glyph(b).text)}'`;
const cond = (char, ctx) => (ctx.length ? `P(${char} | '${text(ctx)}')` : `P(${char})`);
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
