use std::io::{Read, Write};
use std::mem;
use std::slice;

pub struct HiveCPU {
    ram: [u16; 65535],
    reg: [u16; 16],
}

impl HiveCPU {
    pub fn new() -> HiveCPU {
        HiveCPU {
            ram: [0; 65535],
            reg: [0; 16],
        }
    }
    pub fn load_memory(&mut self, reader: &mut Read) {
        unsafe {
            let memory_size = mem::size_of_val(&self.ram);
            let memory_slice = slice::from_raw_parts_mut(
                &mut self.ram as *mut _ as *mut u8,
                memory_size,
            );
            reader.read_exact(memory_slice).unwrap();
        }
    }
    pub fn save_memory(&mut self, writer: &mut Write) {
        unsafe {
            let memory_size = mem::size_of_val(&self.ram);
            let memory_slice = slice::from_raw_parts_mut(
                &mut self.ram as *mut _ as *mut u8,
                memory_size,
            );
            writer.write(memory_slice).unwrap();
        }
    }
    pub fn set_memory(&mut self, address: u16, value: u16) { self.ram[address as usize] = value }
    pub fn get_memory(&self, address: u16) -> u16 { self.ram[address as usize] }
    pub fn get_sp(&self) -> u16 { self.reg[0] }
    pub fn get_pc(&self) -> u16 { self.reg[1] }
    pub fn get_ex(&self) -> u16 { self.reg[2] }
    pub fn get_ia(&self) -> u16 { self.reg[3] }
    pub fn get_a(&self) -> u16 { self.reg[4] }
    pub fn get_b(&self) -> u16 { self.reg[5] }
    pub fn get_c(&self) -> u16 { self.reg[6] }
    pub fn get_x(&self) -> u16 { self.reg[7] }
    pub fn get_y(&self) -> u16 { self.reg[8] }
    pub fn get_z(&self) -> u16 { self.reg[9] }
    pub fn get_i(&self) -> u16 { self.reg[10] }
    pub fn get_j(&self) -> u16 { self.reg[11] }

    pub fn step(&mut self) {

    }
}

