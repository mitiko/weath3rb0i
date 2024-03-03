#!/bin/bash

PS3="Select binary to run: "
binaries=(
    "order0"
    "ac-over-huffman"
)

# TODO: if arg == 1, select the first choice
# TODO: pass release mode

select binaryName in "${binaries[@]}"; do
    cargo run --bin $binaryName
    break
done
