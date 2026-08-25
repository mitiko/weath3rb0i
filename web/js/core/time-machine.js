// Random access to a model's internal state at any bit position.
//
// The JSONL backend is a sparse line index built in a worker: one byte offset per 1024
// lines, 48 KB of index for a 1.1 GB log, about 200 KB read per lookup. Data line k is the
// state entering bit k, so a bit position is a line number.
//
// A wasm model replaying from its last checkpoint implements the same at(), and either may
// answer null for a position it does not cover.

import { emit } from './bus.js';

export class JsonlMachine {
  constructor(file) {
    this.metadata = null;
    this.hint = 'not indexed yet, try again in a moment';
    this.lines = 0;
    this.loaded = 0;
    this.total = file.size;
    this.complete = false;
    this.error = '';
    this.pending = new Map();
    this.seq = 0;

    this.worker = new Worker('js/index-worker.js', { type: 'module' });
    this.worker.onmessage = (e) => this.receive(e.data);
    this.worker.postMessage({ cmd: 'index', file });
  }

  /** State entering bit `pos`, or null if the index does not reach it. */
  at(pos) {
    const id = ++this.seq;
    return new Promise((resolve) => {
      this.pending.set(id, resolve);
      this.worker.postMessage({ cmd: 'line', id, line: pos });
    });
  }

  receive(m) {
    switch (m.type) {
      case 'meta':
        this.metadata = m.metadata;
        break;
      case 'progress':
        this.loaded = m.bytesRead;
        emit('machine:grow');
        break;
      case 'done':
        this.loaded = this.total;
        this.lines = m.lines;
        this.complete = true;
        emit('machine:done');
        break;
      case 'line': {
        const resolve = this.pending.get(m.id);
        if (resolve) { this.pending.delete(m.id); resolve(m.json); }
        break;
      }
      case 'error':
        this.error = m.message;
        emit('machine:done');
        break;
    }
  }

  close() {
    this.worker.terminate();
  }
}
