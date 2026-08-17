// The data that was compressed. Global, and enough on its own for the text grid and the
// context stats: neither needs a model.

import { Layout } from '../layout.js';
import { emit } from './bus.js';
import { hex } from './helpers.js';
import { stream } from './stream.js';

export const source = {
  name: '',
  size: 0,
  loaded: 0,
  sha256: '',
  complete: false,
  bytes: new Uint8Array(0),
  layout: new Layout(),
};

export async function loadSource(file) {
  Object.assign(source, {
    name: file.name,
    size: file.size,
    loaded: 0,
    sha256: '',
    complete: false,
    bytes: new Uint8Array(file.size),
    layout: new Layout(),
  });
  // anything derived from the old bytes is void, and a model has to rescan from zero
  emit('source:load');
  emit('source:grow');

  await stream(file, (chunk, at) => {
    source.bytes.set(chunk, at);
    source.loaded = at + chunk.length;
    source.layout.extend(source.bytes, source.loaded);
    emit('source:grow');
  });

  const digest = await crypto.subtle.digest('SHA-256', source.bytes);
  source.sha256 = Array.from(new Uint8Array(digest), hex).join('');
  source.complete = true;
  emit('source:done');
}
