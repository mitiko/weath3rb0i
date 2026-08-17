// Read a File in chunks. A File is a reference to bytes on disk, so this never buffers the
// whole thing and the first rows appear well before the read finishes.
export async function stream(file, onChunk) {
  const reader = file.stream().getReader();
  let at = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) return;
    onChunk(value, at);
    at += value.length;
  }
}
