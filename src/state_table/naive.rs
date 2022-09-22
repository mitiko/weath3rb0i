use super::{StateTable, StateEntry, impl_state_table_from};

struct NaiveStateTable;
const TABLE: [StateEntry; 3963] = gen_table();

const fn gen_table() -> [StateEntry; 3963] {
    [StateEntry::new(0, [0; 2]); 3963]
}

impl_state_table_from!(NaiveStateTable, TABLE);
