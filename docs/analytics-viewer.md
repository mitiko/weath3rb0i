# Analytics viewer

A browser view over what the coder did, bit by bit. No backend and no build step.

```
cd web && python3 -m http.server 8080
```

Then pick the source file and the two analytics files. A static server is required:
ES modules and workers do not load over `file://`.

## Producing the data

`entropy-hashing-ac-simple` writes two files next to each other:

| File              | Contents                                                    |
|-------------------|-------------------------------------------------------------|
| `analytics.p16`   | one little-endian u16 per bit: the probability it was coded with |
| `analytics.jsonl` | line 0 is the metadata header, data line k is the state entering bit k |

`.p16` is what every colour on screen is computed from, so it loads in one read and
costs 2 bytes per bit. The JSONL is only needed for the state panel and is optional in
the viewer. Both are gitignored.

Any model can produce them: `Analytics::metadata` supplies the header and
`Analytics::log` supplies each line. The header describes the tree, so the viewer
hardcodes no class names.

## The metadata contract

Each node in the header carries `type` as `class/ClassName`, a `description`, optional
`vars`, and optional `children` whose keys match the keys of the logged object.

| class     | primary fields               |
|-----------|------------------------------|
| `model`   | `p` probability, `s` state   |
| `counter` | `p` probability              |
| `history` | `h` hash                     |

Anything else in a logged node is shown as an extra variable. The viewer walks the log
and uses the header only for labels, so a node the header does not describe still
renders.

## Cost and colour

`p` is P(bit == 1) on a 16-bit scale, and the coder treats `p == 0` as 2^-32. The cost
of a bit is the code length of the bit that was actually coded:

```
cost = -log2(p / 65536)        for a 1
cost = -log2(1 - p / 65536)    for a 0
```

Summed over book1 this is 450_361 bytes, exactly what the coder emitted, which is the
end-to-end check the header bar shows as `sum entropy`. A character costs the sum of its
8 bits, so 8 bits per character means no compression at all.

Colours run green to red over 32 buckets. The green half ends at the baseline, editable
in the header bar and 0.586 bits per bit by default. Roughly 57% of book1 lands in the
green half and 6% costs more than 8 bits per character.

## Memory

A `File` is a reference to bytes on disk: `slice()` does no I/O and `stream()` never
buffers the whole file. For book1 the viewer holds 0.8 MB of source, 12 MB of
probabilities and 48 KB of line checkpoints, one per 1024 lines. Individual state lines
are sliced out of the 1.1 GB log on click, roughly 200 KB read per click.

If the log is rewritten while the page holds it, the browser invalidates the handle and
reads fail. Reload and pick the files again.

## Later

The scanning and entropy code sits behind `web/js/analyzer.js` so a wasm-bindgen crate
reusing `Counter`, `Model` and `History` can replace it. That is what extracting L1
histories out of the logged state, and replaying models over them, will need.

### If the text grid ever needs to get cheaper

Draw the rows to a `<canvas>` instead of one span per character. The grid is virtualized
to about 4000 nodes, which is a rounding error next to the probabilities, so there is no
reason to do this yet. If it becomes one, canvas is the lever: it takes the grid to one
node per row, or one for the whole view.

`layout.js` already assigns every byte a row and a column, so mapping a click back to a
byte is arithmetic we have. What it costs is text selection, accessibility, and per-span
hover, all of which come free from real elements today.

Custom elements are the wrong direction here. They cannot reduce the node count, every
instance carries a JS wrapper because its constructor is JS, and a shadow root per
instance costs more than the node. They also put thousands of constructor calls on the
scroll path, where `render()` currently hands the parser one string and runs no JS per
node. They are worth considering only in the dock, where instances number in the tens and
are rebuilt on click rather than on scroll.
