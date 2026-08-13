// Tab health.
//
// `data` is the one number we can state exactly: the bytes this page is deliberately
// holding. Browser memory APIs are all optional and none is portable -- performance.memory
// is Chrome-only, and measureUserAgentSpecificMemory() additionally needs COOP/COEP
// headers that `python3 -m http.server` cannot send -- so they are shown when offered and
// left out otherwise.
//
// There is no CPU API at all. The figure comes from how late a fixed-interval timer fires,
// which tracks main-thread blocking, so it is worth refreshing often: it is most useful
// while scrolling. Memory moves slowly and updates every 10 seconds.

const CPU_TICK = 500;
const MEM_TICK = 10_000;

const mib = (b) => (b / 2 ** 20).toFixed(b < 2 ** 30 ? 0 : 1) + (b < 2 ** 30 ? ' MB' : ' GB');

/** `heldBytes` returns what the page is holding right now. */
export function startMeters(heldBytes) {
  const box = document.getElementById('meters');
  const data = document.getElementById('m-data');
  const dom = document.getElementById('m-dom');
  const heap = document.getElementById('m-heap');
  const cpu = document.getElementById('m-cpu');
  box.hidden = false;

  let last = performance.now();
  setInterval(() => {
    const now = performance.now();
    const elapsed = now - last;
    last = now;
    const busy = Math.min(1, Math.max(0, (elapsed - CPU_TICK) / elapsed));
    cpu.textContent = Math.round(busy * 100) + '%';
  }, CPU_TICK);

  const showMem = async () => {
    data.textContent = mib(heldBytes());
    // Safari and Firefox report no memory at all, so the node count stands in for the
    // other half of the question: whether the virtual scroller is leaking rows
    dom.textContent = document.getElementsByTagName('*').length.toLocaleString('en-US');
    if (performance.memory) {
      heap.textContent = mib(performance.memory.usedJSHeapSize);
    } else if (self.crossOriginIsolated && performance.measureUserAgentSpecificMemory) {
      heap.textContent = mib((await performance.measureUserAgentSpecificMemory()).bytes);
    } else {
      heap.parentElement.hidden = true;
    }
  };
  showMem();
  setInterval(showMem, MEM_TICK);
}
