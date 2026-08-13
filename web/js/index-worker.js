// Builds a sparse line index over the state JSONL so any single line can be sliced back
// out on demand. Colouring reads the .p16 sidecar instead, so nothing on screen waits for
// this: it runs in the background and only the state panel needs it.
//
// A File is a reference to bytes on disk: slice() does no I/O and stream() never buffers
// the whole file, so a 1.2 GB log costs one pass and 48 KB of checkpoints.

const NEWLINE = 0x0a;
const SHIFT = 10;                    // one checkpoint per 1024 lines
const MASK = (1 << SHIFT) - 1;
const PROGRESS_BYTES = 64 << 20;

const decoder = new TextDecoder();

let file = null;
let checkpoints = new Float64Array(1 << 12);
let lastCheckpoint = 0;
let lines = 0;      // file lines seen so far, header included
let maxLineLen = 0;

self.onmessage = async (e) => {
  const msg = e.data;
  try {
    if (msg.cmd === 'index') await index(msg.file);
    else if (msg.cmd === 'line') await sendLine(msg.id, msg.line);
  } catch (err) {
    post({ type: 'error', message: String((err && err.message) || err), id: msg.id });
  }
};

const post = (m) => self.postMessage(m);

async function index(f) {
  file = f;
  checkpoints[0] = 0;

  const reader = f.stream().getReader();
  const headerParts = [];
  let base = 0;        // absolute offset of the current chunk
  let lineStart = 0;
  let nextPost = PROGRESS_BYTES;

  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    if (lines === 0) headerParts.push(value.slice());

    for (let from = 0; ;) {
      const nl = value.indexOf(NEWLINE, from);
      if (nl < 0) break;
      const abs = base + nl;

      // the header line is far longer than the rest and would bloat every fetch window
      if (lines > 0 && abs - lineStart > maxLineLen) maxLineLen = abs - lineStart;
      if (lines === 0) post({ type: 'meta', metadata: JSON.parse(decoder.decode(concat(headerParts, abs))) });

      lines++;
      lineStart = abs + 1;
      if ((lines & MASK) === 0) {
        const ci = lines >> SHIFT;
        if (ci >= checkpoints.length) checkpoints = grow(checkpoints, ci + 1);
        checkpoints[ci] = lineStart;
        lastCheckpoint = ci;
      }
      from = nl + 1;
    }

    base += value.length;
    if (base >= nextPost) {
      nextPost = base + PROGRESS_BYTES;
      post({ type: 'progress', lines, bytesRead: base, total: f.size });
    }
  }

  if (lineStart < f.size) lines++; // a last line with no trailing newline
  post({ type: 'done', lines: Math.max(0, lines - 1), maxLineLen });
}

/** Read data line `k` (0-based, header excluded) and post it parsed. */
async function sendLine(id, k) {
  const fileLine = k + 1;
  const ci = fileLine >> SHIFT;
  const skip = fileLine & MASK;

  if (!file || k < 0 || fileLine >= lines || ci > lastCheckpoint) {
    post({ type: 'line', id, line: k, json: null });
    return;
  }

  const start = checkpoints[ci];
  const stride = (maxLineLen || 255) + 1;
  for (let attempt = 1; attempt <= 3; attempt++) {
    const want = (skip + 1) * stride * attempt + 256;
    const end = Math.min(file.size, start + want);
    const buf = new Uint8Array(await file.slice(start, end).arrayBuffer());

    let pos = 0, ok = true;
    for (let s = 0; s < skip; s++) {
      const nl = buf.indexOf(NEWLINE, pos);
      if (nl < 0) { ok = false; break; }
      pos = nl + 1;
    }
    if (!ok) continue;

    let stop = buf.indexOf(NEWLINE, pos);
    if (stop < 0) {
      if (end < file.size) continue;
      stop = buf.length;
    }
    post({ type: 'line', id, line: k, json: JSON.parse(decoder.decode(buf.subarray(pos, stop))) });
    return;
  }
  post({ type: 'line', id, line: k, json: null });
}

function concat(parts, len) {
  const out = new Uint8Array(len);
  let at = 0;
  for (const part of parts) {
    const take = Math.min(part.length, len - at);
    out.set(part.subarray(0, take), at);
    at += take;
    if (at >= len) break;
  }
  return out;
}

function grow(arr, need) {
  const next = new Float64Array(Math.max(need, Math.ceil(arr.length * 1.5)));
  next.set(arr);
  return next;
}
