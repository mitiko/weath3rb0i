// Rough tab health. Both numbers are estimates: performance.memory is Chrome-only and
// reports the JS heap rather than the whole tab, and there is no CPU API at all, so the
// load figure comes from how late a fixed-interval timer fires.

const TICK = 250;

export function startMeters() {
  const box = document.getElementById('meters');
  const mem = document.getElementById('m-mem');
  const cpu = document.getElementById('m-cpu');
  const fpsEl = document.getElementById('m-fps');
  box.hidden = false;

  let frames = 0;
  const tickFrame = () => { frames++; requestAnimationFrame(tickFrame); };
  requestAnimationFrame(tickFrame);

  let last = performance.now();
  setInterval(() => {
    const now = performance.now();
    const elapsed = now - last;
    last = now;

    const busy = Math.min(1, Math.max(0, (elapsed - TICK) / elapsed));
    cpu.textContent = Math.round(busy * 100) + '%';
    fpsEl.textContent = Math.round(frames * 1000 / elapsed);
    frames = 0;

    const m = performance.memory;
    mem.textContent = m ? (m.usedJSHeapSize / 2 ** 20).toFixed(0) + ' MB' : 'n/a';
  }, TICK);
}
