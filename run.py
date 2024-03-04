#!/usr/bin/env python3

import argparse
import subprocess
from os import environ as env
import os

parser = argparse.ArgumentParser(
    prog="weath3rb0i",
    description="The weath3rb0i binary executor",
    epilog="Run `cargo run` for the default",
)

binaries_path = os.path.join(os.path.dirname(__file__), './src/bin/')
binaries = os.listdir(binaries_path)
binaries.insert(0, 'weath3rb0i')

# Add all parameters globally but only use the ones we require per binary (no checks for extra args)
parser.add_argument("bin", nargs="?", help="Which binary (by id) to run")
parser.add_argument(
    "-r", "--release", action="store_true", help="Whether to compile in release mode"
)
parser.add_argument("-q", "--quiet", action="store_true", help="Hide compiler info")
parser.add_argument("--hsize", type=int, help="Max code length for Huffman tree")

args = parser.parse_args()
if args.bin in binaries:
    args.bin = binaries.index(args.bin)
else:
    try:
        args.bin = int(args.bin) if args.bin is not None else None
    except ValueError:
        args.bin = None

while args.bin is None or args.bin >= len(binaries):
    for i, binary in enumerate(binaries[1:]):
        print(f"{i+1}) {binary}")

    try:
        selected_binary = input("Select binary to run: ")
        if selected_binary in binaries:
            args.bin = binaries.index(selected_binary)
        else:
            args.bin = int(selected_binary)
    except ValueError:
        continue

binary = binaries[args.bin]
cmd = ["cargo", "run"]

if args.quiet:
    cmd.append("--quiet")
if args.release:
    cmd.append("--release")

cmd.extend(["--bin", binary, "--"])

if "FILE" in env:
    cmd.append(env["FILE"])

if "DEBUG" in env:
    print('(dbg) Command:', cmd)

subprocess.run(cmd)
