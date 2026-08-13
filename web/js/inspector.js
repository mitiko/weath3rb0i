// The right-hand dock: the 8 bits of the anchored byte, the encoder state that produced
// them, and how often this character appears in its context.

import { bitAt, costBits, prob, charCost, fixed3, commas } from './analyzer.js';
import { bitBucket } from './color.js';
import { glyph } from './layout.js';

const DEPTH = 3; // context depths beyond the character itself

// What each class in the metadata contract carries as its own primary fields.
const PRIMARY = { model: ['p', 's'], counter: ['p'], history: ['h'] };

// quotes matter too: descriptions from the metadata end up inside title attributes
const ESC_HTML = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' };
const escape = (s) => String(s).replace(/[&<>"]/g, (c) => ESC_HTML[c]);

export class Inspector {
  constructor(state, { getLine, onPick }) {
    this.state = state;
    this.getLine = getLine;
    this.onPick = onPick;
    this.hasState = false; // set once a state file is picked
    this.collected = [];
    this.token = 0;
    this.line = null;

    this.el = {
      hint: document.getElementById('dock-hint'),
      bits: document.getElementById('sec-bits'),
      strip: document.getElementById('bitstrip'),
      bitsSummary: document.getElementById('bits-summary'),
      state: document.getElementById('sec-state'),
      tree: document.getElementById('statetree'),
      ctx: document.getElementById('sec-ctx'),
      ctxStats: document.getElementById('ctxstats'),
      copy: document.getElementById('sec-copy'),
      copyCount: document.getElementById('copy-count'),
      copyStatus: document.getElementById('copy-status'),
    };

    this.el.strip.addEventListener('click', (e) => {
      const bit = e.target.closest('.bit');
      if (bit) this.onPick(+bit.dataset.pos);
    });

    document.getElementById('copy-one').onclick = () => this.copy(this.snapshot());
    document.getElementById('copy-all').onclick = () => this.copy(this.collected);
    document.getElementById('copy-append').onclick = () => {
      this.collected.push(this.snapshot());
      this.el.copyCount.textContent = this.collected.length;
      this.note(`appended, ${this.collected.length} held`);
    };
    document.getElementById('copy-clear').onclick = () => {
      this.collected = [];
      this.el.copyCount.textContent = 0;
      this.note('cleared');
    };
  }

  /** Show everything for an absolute bit position. */
  async show(pos) {
    this.el.hint.hidden = true;
    for (const k of ['bits', 'state', 'ctx', 'copy']) this.el[k].hidden = false;

    this.renderBits(pos);
    this.renderContext(pos >> 3);

    if (!this.hasState) {
      this.el.tree.innerHTML = '<p class="sub">no state file loaded</p>';
      return;
    }
    const token = ++this.token;
    this.el.tree.innerHTML = '<p class="sub">reading line…</p>';
    const json = await this.getLine(pos);
    if (token !== this.token) return; // a newer position won
    this.line = json;
    this.el.tree.innerHTML = json
      ? this.nodeHtml(null, json, this.state.metadata, true)
      : '<p class="sub">not indexed yet — try again in a moment</p>';
  }

  /** Repaint with the current data, used when the baseline or the scan frontier moves. */
  refresh() {
    if (this.state.anchor !== null) this.renderBits(this.state.anchor);
  }

  renderBits(pos) {
    const { bytes, probs, nProbs } = this.state;
    const byte = pos >> 3;
    const base = byte << 3;
    let html = '';
    let total = 0;
    let known = true;

    for (let k = 0; k < 8; k++) {
      const at = base + k;
      const bit = bitAt(bytes, at);
      if (at >= nProbs) {
        known = false;
        html += `<div class="bit eu" data-pos="${at}"><div class="v">${bit}</div>` +
                `<div class="u">—</div><div class="p">—</div><div class="c">—</div></div>`;
        continue;
      }
      const p = probs[at];
      const cost = costBits(p, bit);
      total += cost;
      const on = at === pos ? ' on' : '';
      html += `<div class="bit e${bitBucket(cost)}${on}" data-pos="${at}" ` +
              `title="bit ${at}: p ${p}/65536 = ${prob(p)}, costs ${cost} bits">` +
              `<div class="v">${bit}</div>` +
              `<div class="u">${p}</div>` +
              `<div class="p">${fixed3(prob(p))}</div>` +
              `<div class="c">${fixed3(cost)}</div></div>`;
    }
    this.el.strip.innerHTML = html;

    const g = glyph(bytes[byte]);
    const hex = bytes[byte].toString(16).padStart(2, '0');
    // P(byte) is what the 8 predictions jointly assigned to this character
    const joint = 2 ** -total;
    this.el.bitsSummary.textContent = known
      ? `'${g.text}' 0x${hex} · ${fixed3(total)} bpc · P(byte) ` +
        (joint >= 0.001 ? fixed3(joint) : joint.toExponential(2))
      : `'${g.text}' 0x${hex} · not fully loaded`;
  }

  renderContext(i) {
    // only search the part of the source that has actually streamed in
    const n = this.state.layout.done;
    const bytes = this.state.bytes.subarray(0, n);
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

    this.el.ctxStats.innerHTML = html;
  }

  /**
   * One node of the state tree. The label is the property name the state was logged
   * under, because that is what identifies it in the tree; the class and its description
   * live in the tooltip. Only the root starts open.
   */
  nodeHtml(key, log, meta, open) {
    const type = meta && typeof meta.type === 'string' ? meta.type : '';
    const slash = type.indexOf('/');
    const cls = slash > 0 ? type.slice(0, slash) : '';
    const name = slash > 0 ? type.slice(slash + 1) : type;
    const primary = PRIMARY[cls] || [];
    const children = (meta && meta.children) || {};

    const kids = [];
    const vars = [];
    for (const [k, v] of Object.entries(log)) {
      if (v !== null && typeof v === 'object' && !Array.isArray(v)) kids.push([k, v]);
      else if (!primary.includes(k)) vars.push([k, v]);
    }

    const head = primary
      .filter((k) => k in log)
      .map((k) => `<span class="k">${k}</span> ${fmt(k, log[k])}`)
      .join(' · ');

    const label = key || name || 'state';
    const tip = [cls ? `${name} (${cls})` : name, meta && meta.description].filter(Boolean).join('\n');

    let html = `<details class="node"${open ? ' open' : ''}><summary>`;
    html += `<span class="cname" title="${escape(tip)}">${escape(label)}</span>`;
    if (head) html += ` <span class="prim">${head}</span>`;
    html += '</summary>';

    for (const [k, v] of vars) html += `<div class="kv"><span class="k">${escape(k)}</span> ${fmt(k, v)}</div>`;
    for (const [k, v] of kids) html += this.nodeHtml(k, v, children[k], false);
    return html + '</details>';
  }

  snapshot() {
    const { bytes, probs, nProbs, source, anchor } = this.state;
    const byte = anchor >> 3;
    const bits = [];
    for (let k = 0; k < 8; k++) {
      const at = (byte << 3) + k;
      const bit = bitAt(bytes, at);
      if (at >= nProbs) { bits.push({ bit, p: null }); continue; }
      bits.push({ bit, p: probs[at], prob: r3(prob(probs[at])), cost_bits: r3(costBits(probs[at], bit)) });
    }
    const cost = charCost(probs, nProbs, bytes, byte);
    return {
      source: { name: source.name, size: source.size, sha256: source.sha256 },
      position: { bit: anchor, byte, bit_offset: anchor & 7 },
      char: {
        byte: bytes[byte],
        text: glyph(bytes[byte]).text,
        cost_bits: cost < 0 ? null : r3(cost),
      },
      bits,
      state: this.line,
    };
  }

  async copy(data) {
    try {
      await navigator.clipboard.writeText(JSON.stringify(data, null, 2));
      this.note('copied to clipboard');
    } catch (err) {
      this.note('clipboard blocked: ' + err.message);
    }
  }

  note(msg) {
    this.el.copyStatus.textContent = msg;
  }
}

const r3 = (x) => Math.round(x * 1000) / 1000;

function fmt(key, v) {
  if (Array.isArray(v)) return `[${v.join(', ')}]`;
  if (typeof v === 'string') return `"${escape(v)}"`;
  if (typeof v !== 'number') return escape(String(v));
  if (key === 'p') return `${fixed3(prob(v))} <span class="ctag">(${v})</span>`;
  if (key === 's' || key === 'h') return `${commas(v)} <span class="ctag">0x${v.toString(16)}</span>`;
  return Number.isInteger(v) ? commas(v) : fixed3(v);
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
const pct = (a, b) => (b > 0 ? (100 * a / b).toFixed(3) + '%' : '—');

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
