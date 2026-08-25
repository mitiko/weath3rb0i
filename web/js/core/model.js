// A probability series over the source: one u16 per bit, P(bit == 1) on a 16-bit scale.
// A .p16 file, a wasm model and a hardcoded p all produce the same thing, so nothing
// downstream asks where it came from.

import { costRange } from '../analyzer.js';
import { emit, on } from './bus.js';
import { source } from './source.js';
import { stream } from './stream.js';
import { loadWasm } from './wasm.js';

// bytes per slice handed to the runner, so the page can paint between them
const CHUNK = 1 << 15;

export class Model {
  constructor(name) {
    this.name = name;
    this.size = 0;
    this.loaded = 0;    // u16 entries read
    this.nProbs = 0;    // bits usable: probability in, and the source byte holding it too
    this.entropy = 0;   // summed cost over those bits
    this.complete = false;
    this.probs = new Uint16Array(0);
    this.state = null;  // a time machine, or null until one is loaded

    // a bit needs both halves, so either side arriving can move the frontier
    on('source:grow', () => this.advance());
    on('source:load', () => this.reset());
  }

  // Drop the scan but keep the probabilities and `complete`: the file is still fully read,
  // it just has to be costed against the new bytes from zero.
  reset() {
    this.nProbs = 0;
    this.entropy = 0;
    emit('model:grow', this);
  }

  /** Bits per bit over everything scanned so far. */
  get cr() {
    return this.nProbs ? this.entropy / this.nProbs : 0;
  }

  async load(file) {
    // odd trailing byte cannot form a u16, so round the backing buffer up
    const raw = new Uint8Array(file.size + (file.size & 1));
    this.probs = new Uint16Array(raw.buffer);
    this.size = file.size;
    this.loaded = 0;
    this.nProbs = 0;
    this.entropy = 0;
    this.complete = false;
    emit('model:grow', this);

    await stream(file, (chunk, at) => {
      raw.set(chunk, at);
      this.loaded = (at + chunk.length) >> 1;
      this.advance();
    });

    this.complete = true;
    emit('model:done', this);
  }

  // Run a model built in the browser rather than reading one off disk. The runner keeps
  // its state, so consecutive slices continue one run and the page stays alive between.
  async build(modelSpec, historySpec) {
    const { Runner } = await loadWasm();
    const bytes = source.bytes.subarray(0, source.layout.done);
    if (!bytes.length) throw new Error('load a source file first');

    // throws with the parse error from rust, which the panel shows
    const runner = new Runner(modelSpec, historySpec, bytes);

    this.name = `${modelSpec} over ${historySpec}`;
    this.probs = new Uint16Array(bytes.length * 8);
    this.size = bytes.length * 16;
    this.loaded = 0;
    this.nProbs = 0;
    this.entropy = 0;
    this.complete = false;
    emit('model:grow', this);

    try {
      for (let at = 0; at < bytes.length; at += CHUNK) {
        const end = Math.min(at + CHUNK, bytes.length);
        this.probs.set(runner.run(bytes.subarray(at, end)), at * 8);
        this.loaded = end * 8;
        this.advance();
        await new Promise(requestAnimationFrame);
      }
    } finally {
      runner.free();
    }

    this.complete = true;
    emit('model:done', this);
  }

  advance() {
    const limit = Math.min(this.loaded, source.layout.done * 8);
    if (limit <= this.nProbs) return;
    this.entropy += costRange(this.probs, this.nProbs, limit, source.bytes);
    this.nProbs = limit;
    emit('model:grow', this);
  }
}

export const model = new Model('p16');
