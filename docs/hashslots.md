# Hash slots

With weath3rb0i encoding and decoding are asymmetrical - the encoder has more information and uses it for speed optimizations.
The encoder encodes 4 bits (a nibble) at a time but updates the contexts 1 bit at a time, however contexts are 4-bit aligned, meaning a single look-up is at most **1 cache miss** (both when encoding and decoding).

The **hash table** uses linear (*but actually constant*) probing to map a context to a **slot**.  
A **slot** contains **state information** for the next 4 bits.  
States are 12-bits!

## Probing and cells

The high bits of a hash are used to locate a **cell**.  
Each cell is 96 bytes = 1.5 cache lines.  
Modern processors have *adjacent cache line prefetch* enabled, so the practical cost is negligible.

The cell is divided into 2 parts:
- 4 hashes (6 bytes)
- 4 slots (90 bytes)

The low 12 bits (1.5 bytes) of a hash are used to identify a slot within the cell.  
Cells have 4 slots. This makes probing constant -> 4 comparisons max.  
If the hash doesn't match any of the 4 slots, we have to make a decision.  
If the new context is more valuable than old data, we may overwrite the slot that is least valuable.  
If the new context appears to be an outlier, we may discard it alltogether, returning state 0 and letting the mixer decide how to handle this uncertainty.

For now, as PoC, we overwrite slot 1.  
In the future I want to implement a **delayed overwrite** system that will store temporary statistics for new contexts and *possible outliers* and will track *hot vs cold* slots to choose which one to use.  
Alternatively (and not excluding option 1) this system may **re-route** temporary contexts to cold cells with empty slots left.  
This way the hashtable will further amortize the spread of the hash function.
Hotter cells, that need more than 4 slots, will allocate more slots, and colder cells will donate their slots to hotter ones.

Since we want as many models as possible, we want to fill a fixed-size hashtable with as many entries as possible, operating on average above 80% capacity.  
To dynamically reallocate slots becomes an allocator problem and we can apply allocator solutions to it.

## Slots

Each slot contains 15 states:
- 1 for the first bit,
- 2 for the second,
- 4 for the third,
- and 8 for the last.

Each state is 12-bits (1.5 bytes) making 1 slot = 22.5 bytes.  
In practice, the memory of the 4 slots are concatenated into a 90-byte window.

The `Slot` struct is constructed as a *temporary* index into the cell's memory.  
It's used to get and set any of the 15 states by a `bit_id` and `nib_ctx`

The `bit_id` parameter refers to a bit position in a nibble.
Possible values are `[0-3]`.

The `nib_ctx` paramater refers to the nibble context - the bits we have access to.  
For example, if `bit_id = 2` this means we have access to the previous 2 bits,
and they are stored in Big Endian in `nib_ctx`

### Example

To understand how slots index the cell's memory, here's a table of what byte will be indexed.
`rel_idx` is the relative index - within the slot.  
`abs_idx` is the absolute index - within the cell.  
Since states are 12-bit, in 2 consecutive bytes, the state will be stored either in the lower bits and need be masked, or in the higher bits and need be shifted.

The formula is:  
`rel_idx = (1 << bit_id) - 1 + nib_ctx`  
`abs_idx = ((rel_idx + slot.id * 15) * 3) >> 1`  
`parity = ((rel_idx + slot.id * 15) * 3) & 1`


| `slot.id` | `bit_id` | `nib_ctx` | `rel_idx` | `abs_idx` | parity |
|-----------|----------|-----------|-----------|-----------|--------|
| 0         | 0        | 0000      | 0         | 0         | shift  |
| 0         | *1*      | 000**0**  | 1         | 1         | mask   |
| 0         | *1*      | 000**1**  | 2         | 3         | shift  |
| 0         | 2        | 00**00**  | 3         | 4         | mask   |
| 0         | 2        | 00**01**  | 4         | 6         | shift  |
| 0         | 2        | 00**10**  | 5         | 7         | mask   |
| 0         | 2        | 00**11**  | 6         | 9         | shift  |
| 0         | *3*      | 0**000**  | 7         | 10        | mask   |
| 0         | *3*      | 0**001**  | 8         | 12        | shift  |
| 0         | *3*      | 0**010**  | 9         | 13        | mask   |
| 0         | *3*      | 0**011**  | 10        | 15        | shift  |
| 0         | *3*      | 0**100**  | 11        | 16        | mask   |
| 0         | *3*      | 0**101**  | 12        | 18        | shift  |
| 0         | *3*      | 0**110**  | 13        | 19        | mask   |
| 0         | *3*      | 0**111**  | 14        | 21        | shift  |
|           |          |           |           |           |        |
| 1         | 0        | 0000      | 0         | 22        | mask   |
| 1         | *1*      | 000**0**  | 1         | 24        | shift  |
| 1         | *1*      | 000**1**  | 2         | 25        | mask   |
| 1         | 2        | 00**00**  | 3         | 27        | shift  |
| 1         | 2        | 00**01**  | 4         | 28        | mask   |
| 1         | 2        | 00**10**  | 5         | 30        | shift  |
| 1         | 2        | 00**11**  | 6         | 31        | mask   |
| 1         | *3*      | 0**000**  | 7         | 33        | shift  |
| 1         | *3*      | 0**001**  | 8         | 34        | mask   |
| 1         | *3*      | 0**010**  | 9         | 36        | shift  |
| 1         | *3*      | 0**011**  | 10        | 37        | mask   |
| 1         | *3*      | 0**100**  | 11        | 39        | shift  |
| 1         | *3*      | 0**101**  | 12        | 40        | mask   |
| 1         | *3*      | 0**110**  | 13        | 42        | shift  |
| 1         | *3*      | 0**111**  | 14        | 43        | mask   |
|           |          |           |           |           |        |
| 2         | 0        | 0000      | 0         | 45        | shift  |
| 2         | *1*      | 000**0**  | 1         | 46        | mask   |
| 2         | *1*      | 000**1**  | 2         | 48        | shift  |
| 2         | 2        | 00**00**  | 3         | 49        | mask   |
| 2         | 2        | 00**01**  | 4         | 51        | shift  |
| 2         | 2        | 00**10**  | 5         | 52        | mask   |
| 2         | 2        | 00**11**  | 6         | 54        | shift  |
| 2         | *3*      | 0**000**  | 7         | 55        | mask   |
| 2         | *3*      | 0**001**  | 8         | 57        | shift  |
| 2         | *3*      | 0**010**  | 9         | 58        | mask   |
| 2         | *3*      | 0**011**  | 10        | 60        | shift  |
| 2         | *3*      | 0**100**  | 11        | 61        | mask   |
| 2         | *3*      | 0**101**  | 12        | 63        | shift  |
| 2         | *3*      | 0**110**  | 13        | 64        | mask   |
| 2         | *3*      | 0**111**  | 14        | 66        | shift  |
|           |          |           |           |           |        |
| 3         | 0        | 0000      | 0         | 67        | mask   |
| 3         | *1*      | 000**0**  | 1         | 69        | shift  |
| 3         | *1*      | 000**1**  | 2         | 70        | mask   |
| 3         | 2        | 00**00**  | 3         | 72        | shift  |
| 3         | 2        | 00**01**  | 4         | 73        | mask   |
| 3         | 2        | 00**10**  | 5         | 75        | shift  |
| 3         | 2        | 00**11**  | 6         | 76        | mask   |
| 3         | *3*      | 0**000**  | 7         | 78        | shift  |
| 3         | *3*      | 0**001**  | 8         | 79        | mask   |
| 3         | *3*      | 0**010**  | 9         | 81        | shift  |
| 3         | *3*      | 0**011**  | 10        | 82        | mask   |
| 3         | *3*      | 0**100**  | 11        | 84        | shift  |
| 3         | *3*      | 0**101**  | 12        | 85        | mask   |
| 3         | *3*      | 0**110**  | 13        | 87        | shift  |
| 3         | *3*      | 0**111**  | 14        | 88        | mask   |


Slot 0 has bytes `[0-21]` and _**high** nibble_ of `byte 22`  
Slot 1 has _**low** nibble_ of `byte 22` and bytes `[23-44]`  
Slot 2 has bytes `[45-66]` and _**high** nibble_ of `byte 67`  
Slot 3 has _**low** nibble_ of `byte 67` and bytes `[68-89]`  
