// The model's internals entering the anchored bit, read out of the time machine.
//
// The tree is generic: any JSON object renders. The metadata header only supplies labels
// and descriptions, so a node it does not describe still shows up.

import { prob } from '../analyzer.js';
import { on } from '../core/bus.js';
import { commas, escape, fixed3 } from '../core/helpers.js';
import { model } from '../core/model.js';
import { view } from '../core/view.js';

// What each class in the metadata contract carries as its own primary fields.
const PRIMARY = { model: ['p', 's'], counter: ['p'], history: ['h'] };

export class EncoderState extends HTMLElement {
  connectedCallback() {
    this.innerHTML = '<h2>encoder state</h2><div id="statetree"></div>';
    this.tree = this.querySelector('#statetree');
    this.token = 0;

    this.ac = new AbortController();
    on('anchor', () => this.render(), this.ac.signal);
    this.render();
  }

  disconnectedCallback() {
    this.ac.abort();
  }

  async render() {
    this.hidden = view.anchor === null;
    if (this.hidden) return;

    const machine = model.state;
    if (!machine) {
      this.tree.innerHTML = '<p class="sub">no model running and no state file loaded</p>';
      return;
    }

    const token = ++this.token;
    this.tree.innerHTML = '<p class="sub">reading line...</p>';
    const json = await machine.at(view.anchor);
    if (token !== this.token) return; // a newer position won

    this.tree.innerHTML = json
      ? nodeHtml(null, json, machine.metadata, true)
      : `<p class="sub">${machine.hint}</p>`;
  }
}

// One node of the tree. The label is the property the state was logged under, since that
// is what identifies it; the class and description go in the tooltip. Only the root opens.
function nodeHtml(key, log, meta, open) {
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
    .join(' - ');

  const label = key || name || 'state';
  const title = cls ? `${name} (${cls})` : name;
  const tip = [title, meta && meta.description].filter(Boolean).join('\n');

  let html = `<details class="node"${open ? ' open' : ''}><summary>`;
  html += `<span class="cname" title="${escape(tip)}">${escape(label)}</span>`;
  if (head) html += ` <span class="prim">${head}</span>`;
  html += '</summary>';

  for (const [k, v] of vars) html += `<div class="kv"><span class="k">${escape(k)}</span> ${fmt(k, v)}</div>`;
  for (const [k, v] of kids) html += nodeHtml(k, v, children[k], false);
  return html + '</details>';
}

function fmt(key, v) {
  if (Array.isArray(v)) return `[${v.join(', ')}]`;
  if (typeof v === 'string') return `"${escape(v)}"`;
  if (typeof v !== 'number') return escape(String(v));
  if (key === 'p') return `${fixed3(prob(v))} <span class="ctag">(${v})</span>`;
  if (key === 's' || key === 'h') return `${commas(v)} <span class="ctag">0x${v.toString(16)}</span>`;
  return Number.isInteger(v) ? commas(v) : fixed3(v);
}

customElements.define('x-encoder-state', EncoderState);
