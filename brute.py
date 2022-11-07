#!/usr/bin/python3

# get all but 1 byte from book1
import shutil
import filecmp
import os

book1 = "/data/calgary/book1"
# copy compressed file without last byte
comp_file = "book1.bin"
base_file = "book1-mod.bin"
comp_size = os.stat(comp_file).st_size
os.system(f"dd if=book1.bin of=book1-mod.bin bs={comp_size - 1} count=1")

for byte in range(256):
    id = format(byte, "#03")
    dest = f"book1-py-{id}.bin"
    shutil.copy(base_file, dest)
    # append byte to file
    with open(dest, "ab") as file:
        file.write(byte.to_bytes(1, byteorder="big"))
    # try to decode
    os.system(f"target/release/weath3rb0i d {dest} > /dev/null")
    # compare with initial file
    orig = f"book1-py-{id}.orig"
    # if they match, print byte
    # if they don't, delete the files
    if filecmp.cmp(orig, book1):
        print(f"byte {byte} was bruteforced")
        os.system(f"rm -rf {orig}")
    else:
        os.system(f"rm -rf {dest} {orig}")

