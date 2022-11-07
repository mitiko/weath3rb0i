#!/usr/bin/python3

def printb(x): print(format(x, "#034b"))
def printb8(x): print(format(x, "#010b"))

# book2:
# rev = 0
# bitq = 22
# cnt = 5
# lowx = 788749607

# book1:
rev = 1
bitq = 54
cnt = 3
lowx = 1310222768

t = bitq & ((1 << cnt) - 1) # last cnt bits

print(f"cnt is:\t{cnt}\t=", end='')
printb8(cnt)
print(f"t is:\t{t}\t=", end='')
printb8(t)
print(f"rev is:\t{rev}\t=", end='')
printb(rev)
print(f"lowx is:\t=", end='')
printb(lowx)

# get the top 8-cnt bits from lowx
flush_bits = (lowx >> 24) >> cnt
print("flush bits:\t=", end='')
printb8(flush_bits)

flush_byte = (t << (8 - cnt)) | flush_bits
print("flush byte:\t=", end='')
printb8(flush_byte)
print(flush_byte)

print(hex(flush_byte))
# the flush is correct

# check the state at writes

print("\n\n\n------")
printb8(0xcc)
printb8(0xc9)
print("\n------")
max = (1 << 32) - 1
print(f"(lowx << 0) = {lowx << 0}  = ", end='')
printb((lowx << 0) & max)
print(f"(lowx << 1) = {lowx << 1}  = ", end='')
printb((lowx << 1) & max)
print(f"(lowx << 2) = {lowx << 2}  = ", end='')
printb((lowx << 2) & max)
print(f"(lowx << 3) = {lowx << 3} = ", end='')
printb((lowx << 3) & max)
print(f"(lowx << 4) = {lowx << 4} = ", end='')
printb((lowx << 4) & max)
print(f"(lowx << 5) = {lowx << 5} = ", end='')
printb((lowx << 5) & max)

print("\n\n------")

(x1, x2) = (1310222768, 3645124919)
printb(x1)
printb(x2)

print("not", end="")
# printb8((204 << 3) & 255)
printb8(204)
for byte in range(205, 214):
    print("---", end="")
    # printb8((byte << 3) & 255)
    printb8(byte)

print("not", end="")
# printb8((214 << 3) & 255)
printb8(214)
