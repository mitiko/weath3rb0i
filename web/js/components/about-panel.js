// What this is and what the tabs hold. Static, so it renders once and subscribes to nothing.

export class AboutPanel extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `
      <h2>about</h2>
      <p class="sub">A compression debugger. The coder writes down what its model predicted
        for every bit; this reads that back and shows where the bits went. Any model works:
        an order-0 counter, a context mixer, a neural net, a hardcoded p.</p>

      <h2>inputs</h2>
      <p class="sub">The source file that was compressed. A <b>.p16</b>, one u16 per bit,
        the probability it was coded with. Optionally a <b>.jsonl</b>, the model's state
        entering each bit, read one line at a time out of a sparse index.</p>

      <h2>source</h2>
      <p class="sub">The files and what they add up to. LTCB position ranks the ratio
        against the <a href="https://www.mattmahoney.net/dc/text.html">LTCB</a> enwik9
        results: a different corpus, so it places the number rather than scoring it.</p>

      <h2>state</h2>
      <p class="sub">The eight bits of one character, each with its probability and cost,
        and the model internals entering the bit you picked.</p>

      <h2>analysis</h2>
      <p class="sub">How predictable the character was from the text alone, counted straight
        out of the source with no model involved.</p>

      <h2>wasm, later</h2>
      <p class="sub">Run the model in the browser and there are no sidecars to load: the
        probabilities come out of wasm memory directly, and exact state at any position
        comes from replaying the last checkpoint instead of a 1 GB log. Two models over one
        text then becomes two runs to compare.</p>

      <h2>credits</h2>
      <p class="sub">Author Dimitar Rusev
        <a href="https://github.com/mitiko">@mitiko</a><br>
        Viewer generated with Opus 5</p>`;
  }
}

customElements.define('x-about-panel', AboutPanel);
