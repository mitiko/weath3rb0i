# weath3rb0i - a lightweight-ish CM compressor

<code>// TODO: general info about the compressor. </code>

## Architecture

> The Predictor is Models + Mixer.

The idea here is that the predictor shouldn't be tied to the specific implementations of models or a mixer and just operate on the interfaces they provide (the traits they implement).

> A Model is stats + context.

Each model has its own seperate statistics _"database"_ and a context to _query_ it.

Bigger prefix models may use a shared hashtable in the future.

## Prediction vs Updates + Asymmetric operation

Inspired by the hashtable implementation, models have 2 seperate interfaces for encoding and decoding.

When compressing we can take advantage of the fact that the next bits are known and get 4 predictions for the cost of 1.  
On decompression we also group stats into nibble trees (and the memory call cost is the same) but there's a dependency on the bits.

Basically predictions are bitwise but updates are nibblewise. On decoding we emulate nibblewise updates by 4 bitwise updates with no extra memory calls.

## Future

v1.0 will feature:
- bitwise entropy coding (ABS?) with nibblewise context updates
- 16-bit predictions
- a 12-bit (tunable) state table
- a 96 byte cell hashmap for cache-line optimization
- written in 100% safe rust
- ability to outsource collected stats to a better NN for testing (and parameter tuning)
- APM mixers

Wishing to futher experiment with:
- entropy based hashing
- asymmetric compression (not just ANS)
- MT
- CM parsing optimization
