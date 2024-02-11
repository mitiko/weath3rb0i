#!/bin/bash

PS3="Select binary to run: "
binaries=(
    "entropy-hashing-package-merge-search"
    # "entropy-hashing-huffman-search"
)

# TODO: if arg == 1, select the first choice
# TODO: pass release mode

select binaryName in "${binaries[@]}"; do
    cargo run --bin $binaryName
    break
done
