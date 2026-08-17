// Tab health, shown next to the scan progress.
//
// There is no CPU API. The figure is how late a fixed-interval timer fires, which tracks
// main-thread blocking and is most useful while scrolling. Memory has no portable API at
// all, so that counter reads N/A.

const CPU_TICK = 500;

export function startMeters() {
  const cpu = document.getElementById('m-cpu');

  let last = performance.now();
  setInterval(() => {
    const now = performance.now();
    const elapsed = now - last;
    last = now;
    const busy = Math.min(1, Math.max(0, (elapsed - CPU_TICK) / elapsed));
    cpu.textContent = Math.round(busy * 100) + '%';
  }, CPU_TICK);
}
