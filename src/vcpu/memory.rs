use std::io::{Read, Write};
use std::mem;
use std::slice;

pub struct Memory {
    data: [u16; 65536],
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            data: [0; 65536],
        }
    }
    pub fn load_memory(&mut self, reader: &mut Read) {
        unsafe {
            let memory_size = mem::size_of_val(&self.data);
            let memory_slice = slice::from_raw_parts_mut(
                &mut self.data as *mut _ as *mut u8,
                memory_size,
            );
            reader.read_exact(memory_slice).unwrap();
        }
    }
    pub fn save_memory(&mut self, writer: &mut Write) {
        unsafe {
            let memory_size = mem::size_of_val(&self.data);
            let memory_slice = slice::from_raw_parts_mut(
                &mut self.data as *mut _ as *mut u8,
                memory_size,
            );
            writer.write(memory_slice).unwrap();
        }
    }
    pub fn set_memory(&mut self, address: u16, value: u16) {
        self.data[address as usize] = value
    }
    pub fn get_memory(&self, address: u16) -> u16 {
        self.data[address as usize]
    }
}

#[cfg(test)]
mod tests {
    use vcpu::memory::Memory;
    use rand::{Rng, SeedableRng, XorShiftRng};
    use std::io::Cursor;

    #[test]
    pub fn test_save_load_memory() {
        // Create our Memory and external buffers
        let mut output: [u8; 131072] = [0; 131072];
        let mut input: [u8; 131072] = [0; 131072];
        let mut memory = Memory::new();

        // Fill our input Buffer
        XorShiftRng::from_seed([1; 4]).fill_bytes(&mut input[..]);

        // Load our input into Memory
        memory.load_memory(&mut Cursor::new(&mut input[..]));

        // Save our memory to output
        memory.save_memory(&mut Cursor::new(&mut output[..]));

        // Compare buffers
        assert_eq!(&input[..], &output[..]);
    }
}