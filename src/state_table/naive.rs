use std::char::MAX;

use super::{StateTable, StateEntry, impl_state_table_from};

// TODO: Docs
// notes are from 22.09.2022
pub struct NaiveStateTable;

const MAX_LEVEL: usize = 44;
const SUBTABLE_SIZE: usize = MAX_LEVEL * (MAX_LEVEL + 1) / 2; // 990
const SIZE: usize = 3 + 4 * SUBTABLE_SIZE; // 3963 <= 4096 = 1 << 12
const TABLE: [StateEntry; SIZE] = gen_table();

const HALF: u16 = 1 << (u16::BITS - 1);
const OFFSET: u16 = SUBTABLE_SIZE as u16;

const fn gen_table() -> [StateEntry; SIZE] {
    let mut t = [StateEntry::new(0, [0; 2]); SIZE];
    let a = 3;
    let b = a + OFFSET;
    let c = a + 2 * OFFSET;
    let d = a + 3 * OFFSET;

    // Entry nodes
    t[0] = StateEntry::new(HALF, [1, 2]);
    t[1] = StateEntry::new(HALF, [a, b]);
    t[2] = StateEntry::new(HALF, [c, d]);
    let a_reg = a as usize;
    let b_reg = b as usize;
    let c_reg = c as usize;
    let d_reg = d as usize;

    // Get auxiliary table
    let at = gen_auxiliary_table();

    // Connect nodes as the auxiliary dictates
    let mut i = 0;
    while i < SUBTABLE_SIZE {
        let next = at[i].next;
        let p = at[i].prob;

        t[a_reg + i] = StateEntry::new(p, [a + next[0], b + next[1]]);
        t[b_reg + i] = StateEntry::new(p, [c + next[0], d + next[1]]);
        t[c_reg + i] = StateEntry::new(p, [a + next[0], b + next[1]]);
        t[d_reg + i] = StateEntry::new(p, [c + next[0], d + next[1]]);
        i+=1;
    }

    t
}

const fn gen_auxiliary_table() -> [StateEntry; SUBTABLE_SIZE] {
    let mut at = [StateEntry::new(0, [0; 2]); SUBTABLE_SIZE];

    // const_for loops not yet implemented in nightly
    let mut level = 1;
    let mut filled = 0; // number of nodes filled up until level-1
    while level <= MAX_LEVEL {
        // at level i, we need to fill i nodes
        let mut node = 0;
        while node < level {
            let prob = calc_prob(node as u16, level as u16 - 1);
            let next = get_next_nodes(level, filled, node);
            debug_assert!(next[0] < OFFSET && next[1] < OFFSET);

            at[filled + node] = StateEntry::new(prob, next);
            node += 1;
        }

        filled += level;
        level += 1;
    }
    // TODO: Hmm, why doesn't the debug_assert_eq macro work in const?
    // Print auxiliary table as nodes?

    at
}

const fn get_next_nodes(level: usize, filled: usize, node: usize) -> [u16; 2] {
    // TODO: Handle last level connections
    if level == MAX_LEVEL {
        // find node that has similiar probability (discard half the bits)
        let next_level = ((level + 2) / 2) - 1; // = ((level-1+1+2)/2)-2+1 = 22
        let next_node_idx = ((node + 2) / 2) - 1; // if bit = 1
        debug_assert!(next_node_idx < next_level);

        let curr_node = (filled + node) as u16;
        let next_node = (next_node_idx + (next_level - 1) * next_level / 2) as u16;
        let mut next = [next_node, next_node];

        if node == 0 { next[0] = curr_node; }
        if node == level - 1 { next[1] = curr_node; }
        return next;
    }

    let next_node = (filled + level + node) as u16;
    let next = [next_node, next_node + 1];

    next
}

// TODO: Try rounding (instead of flooring)
const fn calc_prob(count: u16, total: u16) -> u16 {
    // Same as Counter prob calculation
    let c1 = count as u64;
    let t = total as u64;
    let p = (1 << 16) * (c1 + 1) / (t + 2);
    p as u16
}

impl_state_table_from!(NaiveStateTable, TABLE);

// // Add `weath3rb0i::state_table::naive::print();` to main
// // Resolve dependencies - make the state_table module public - `pub mod state_table`
// // Run `cargo run > state_table.csv`
// // Delete error message at the end of the file
// pub fn print() {
//     println!("state,tr0,tr1,prob");
//     for i in 0..SIZE {
//         let s = i as u16;
//         let s0 = NaiveStateTable::next(s, 0);
//         let s1 = NaiveStateTable::next(s, 1);
//         let p = NaiveStateTable::p(s);

//         println!("{},{},{},{}", s, s0, s1, p);
//     }
// }
