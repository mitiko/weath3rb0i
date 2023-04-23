# weath3rb0i - lightweight CM text compressor

weath3rb0i is a CM single-file text compressor aimed at delivering high
compression ratios with novel modeling approaches.

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

`weath3rb0i <Action> <Path>`

**Action**: c (compress), d (decompress), t (test = c + d)
**Path** can be a single file or a directory
Directories are shallow traversed and each file is compressed individually

## License

GPLv3.0
