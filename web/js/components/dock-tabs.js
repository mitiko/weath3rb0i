// The dock header. Which tab is showing is an attribute on the dock, so the panels
// themselves know nothing about tabs and CSS does the hiding.

import { on } from '../core/bus.js';
import { setTab, view } from '../core/view.js';

const TABS = ['source', 'state', 'analysis', 'about'];

export class DockTabs extends HTMLElement {
  connectedCallback() {
    this.replaceChildren(...TABS.map((name) => {
      const tab = document.createElement('span');
      tab.className = 'tab';
      tab.dataset.tab = name;
      tab.textContent = name;
      tab.onclick = () => setTab(name);
      return tab;
    }));

    this.ac = new AbortController();
    on('tab', () => this.render(), this.ac.signal);
    this.render();
  }

  disconnectedCallback() {
    this.ac.abort();
  }

  render() {
    this.closest('#dock').dataset.tab = view.tab;
    for (const tab of this.children) tab.classList.toggle('on', tab.dataset.tab === view.tab);
  }
}

customElements.define('x-dock-tabs', DockTabs);
