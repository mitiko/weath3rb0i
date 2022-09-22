pub mod naive;

pub trait StateTable {    
    fn next(state: u16, bit: u8) -> u16;
    fn next4(states: [u16; 4], nib: u8) -> [u16; 4] {[
        Self::next(states[0], nib >> 3),
        Self::next(states[1], (nib >> 2) & 1),
        Self::next(states[2], (nib >> 1) & 1),
        Self::next(states[3], nib & 1)
    ]}

    fn p(state: u16) -> u16;
    fn p4(states: [u16; 4]) -> [u16; 4] {
        [Self::p(states[0]), Self::p(states[1]), Self::p(states[2]), Self::p(states[3])]
    }
}

#[derive(Clone, Copy)]
pub struct StateEntry {
    prob: u16,
    next: [u16; 2]
}

impl StateEntry {
    const fn new(prob: u16, next: [u16; 2]) -> Self {
        Self { prob, next }
    }
}

// TODO: Derive macro for trait StateTable, with params - TABLE
macro_rules! impl_state_table_from {
    ($state_table_name:ident, $table:ident) => {
        // struct $state_table_name;

        impl StateTable for $state_table_name {
            fn next(state: u16, bit: u8) -> u16 {
                $table[usize::from(state)].next[usize::from(bit)]
            }
        
            fn p(state: u16) -> u16 {
                $table[usize::from(state)].prob
            }
        }
    };
}

pub(crate) use impl_state_table_from;
