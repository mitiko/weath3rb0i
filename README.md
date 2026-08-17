# weath3rb0i

Experimental CM single-file text compressor aimed at
high compression ratio with novel modeling approaches.

Learning the [bitter lesson](http://www.incompleteideas.net/IncIdeas/BitterLesson.html) and
implementing the basic compressor building block to easily experiment & compare model -> learn.

- `src/` has the library with models, mixers, entropy coders, etc.
- `src/bin/` has examples & experimental coders
- `web/` is a time-machine compressor debugger / web viewer
  - uses same lib models & components compiled to WASM
  - runs statically in browser - no server

wishlist / todo:
- state table gen 12-bit
- 12-bit hash table (impl WIP)
- APM mixers
- pre-processing (dictionaries, flag streams)
- tokenization & attention

## License

GPLv3.0
