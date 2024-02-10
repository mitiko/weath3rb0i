#!/bin/bash

PS3="Select binary to run: "
binaries=(
    "entropy-hashing-package-merge-search"
    # "entropy-hashing-huffman-search"
)

select binaryName in "${binaries[@]}"; do
    cargo run --bin $binaryName
    break
done
