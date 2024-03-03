# weath3rb0i - lightweight CM text compressor

weath3rb0i is an experimental CM single-file text compressor aimed at delivering
high compression ratios with novel modeling approaches.

The goal for v1.0:
- 12-bit state table
- written in 100% safe rust
- output stats from contexts for use by external neural nets
- APM mixers

Wishing to futher experiment with:
- entropy based hashing
- CM parsing optimization
- MT

## Usage

The main binary is WIP but there are many test-only scenarios you can execute
with:

`./run.py <binary> <...options>`

<!-- Main binary: -->
<!--
`weath3rb0i <Action> <Path>`
**Action**: c (compress), d (decompress), t (test = c + d)
**Path** can be a single file or a directory
Directories are shallow traversed and each file is compressed individually
-->

## License

GPLv3.0

Please contact me [@x_mitiko](https://twitter.com/x_mitiko) if you need a copy under a different license.
