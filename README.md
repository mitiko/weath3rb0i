# weath3rb0i - a lightweight-ish CM compressor

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
