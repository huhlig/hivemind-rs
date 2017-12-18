///
///
///
///

type SlotId = usize;

const chunk_size: usize = 256;

pub struct SlotMap<T> {
    object_table: Vec<T>,
    free_list: Vec<usize>,
}