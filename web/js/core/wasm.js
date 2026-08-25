// The wasm module, loaded once on first use.
//
// wasm-pack --target web emits an ES module whose default export fetches the .wasm, so
// there is nothing to bundle and nothing to configure. Everything past this file talks to
// `Runner` and never to the module itself.

let ready = null;

export function loadWasm() {
  if (!ready) {
    ready = import('../../wasm/weath3rb0i.js').then(async (mod) => {
      await mod.default();
      return mod;
    });
  }
  return ready;
}
