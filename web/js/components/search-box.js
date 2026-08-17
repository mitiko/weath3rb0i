// Find bar over the text, opened with cmd-F and closed with escape.
//
// Typing only recounts and re-marks; the view moves when you ask it to with next or
// previous, so the page does not jump around under a half-typed query.

import { on } from '../core/bus.js';
import { commas } from '../core/helpers.js';
import { clear, search, setQuery, step } from '../core/search.js';
import { source } from '../core/source.js';

const DEBOUNCE = 120;

export class SearchBox extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `
      <input type="search" placeholder="find bytes" spellcheck="false">
      <span class="count"></span>
      <button class="prev" title="previous match, shift-enter">&#8593;</button>
      <button class="next" title="next match, enter">&#8595;</button>`;

    this.input = this.querySelector('input');
    this.count = this.querySelector('.count');
    this.hidden = true;

    this.ac = new AbortController();
    const { signal } = this.ac;
    on('search', () => this.render(), signal);

    this.input.addEventListener('input', () => {
      clearTimeout(this.timer);
      this.timer = setTimeout(() => setQuery(this.input.value), DEBOUNCE);
    }, { signal });

    this.input.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') step(e.shiftKey ? -1 : 1);
      if (e.key === 'Escape') this.close();
    }, { signal });

    this.querySelector('.prev').addEventListener('click', () => step(-1), { signal });
    this.querySelector('.next').addEventListener('click', () => step(1), { signal });

    addEventListener('keydown', (e) => {
      if (e.key !== 'f' || !(e.metaKey || e.ctrlKey) || !source.size) return;
      e.preventDefault();
      this.hidden = false;
      this.input.focus();
      this.input.select();
    }, { signal });

    this.render();
  }

  disconnectedCallback() {
    this.ac.abort();
  }

  close() {
    this.input.value = '';
    clear();
    this.hidden = true;
  }

  render() {
    const total = search.matches.length;
    if (!search.needle.length) this.count.textContent = '';
    else if (!total) this.count.textContent = 'no matches';
    else if (search.at < 0) this.count.textContent = `${commas(total)} matches`;
    else this.count.textContent = `${commas(search.at + 1)} / ${commas(total)}`;
    this.classList.toggle('empty', Boolean(search.needle.length) && !total);
  }
}

customElements.define('x-search-box', SearchBox);
