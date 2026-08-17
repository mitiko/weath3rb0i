// Tab health, shown next to the scan progress.
//
// There is no CPU API at all. The figure comes from how late a fixed-interval timer fires,
// which tracks main-thread blocking, so it is worth refreshing often: it is most useful
// while scrolling.
//
// Memory has no reading to give. Every browser API for it is optional and none is
// portable -- performance.memory is Chrome-only, and measureUserAgentSpecificMemory()
// additionally needs COOP/COEP headers that `python3 -m http.server` cannot send -- so the
// counter sits at N/A rather than reporting a number on one browser and nothing elsewhere.

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
