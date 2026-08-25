// A time machine backed by a live model rather than a log.
//
// The runner only goes forward, so asking for a position behind it means rebuilding and
// replaying from the start. That is fine for the file sizes this viewer is for. What is
// kept is the analytics at each position visited, not the state that produced them: a
// snapshot of the state would be megabytes, a snapshot of the analytics is a few hundred
// bytes.

import { source } from './source.js';
import { loadWasm } from './wasm.js';

export class Checkpoints {
  constructor(Runner, modelSpec, historySpec) {
    this.Runner = Runner;
    this.modelSpec = modelSpec;
    this.historySpec = historySpec;
    this.taken = new Map(); // bit position -> analytics, every one ever visited
    this.metadata = null;
    this.hint = 'run the model to this position, on the model tab';
    this.runner = null;
  }

  static async open(modelSpec, historySpec) {
    const { Runner } = await loadWasm();
    const cp = new Checkpoints(Runner, modelSpec, historySpec);
    cp.rebuild();
    return cp;
  }

  /** A fresh runner at bit 0. Throws the parse error from rust. */
  rebuild() {
    this.runner?.free();
    this.runner = new this.Runner(this.modelSpec, this.historySpec, this.bytes());
    if (!this.metadata) this.metadata = JSON.parse(this.runner.metadata());
  }

  bytes() {
    return source.bytes.subarray(0, source.layout.done);
  }

  /** The bit the model is about to code. */
  get position() {
    return this.runner ? this.runner.at() : 0;
  }

  /**
   * Encode up to `bit` and record the state entering it. Returns the predictions made on
   * the way and the bit they start at, so the caller can file them.
   */
  advance(bit) {
    if (bit < this.position) this.rebuild();
    const from = this.position;
    const probs = this.runner.run_to(this.bytes(), bit);
    this.taken.set(bit, JSON.parse(this.runner.analytics()));
    return { from, probs };
  }

  /** The state panel's interface: null for a position not visited yet. */
  at(pos) {
    return this.taken.get(pos) ?? null;
  }

  close() {
    this.runner?.free();
    this.runner = null;
  }
}
