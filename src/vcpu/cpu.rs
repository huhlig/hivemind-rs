use std::io::{Read, Write};
use vcpu::memory::Memory;

pub struct HiveCPU {
    memory: Memory,
    reg: [u16; 16],
}

impl HiveCPU {
    pub fn new() -> HiveCPU {
        HiveCPU {
            memory: Memory::new(),
            reg: [0; 16],
        }
    }
    pub fn load_memory(&mut self, reader: &mut Read) {
        self.memory.load_memory(reader)
    }
    pub fn save_memory(&mut self, writer: &mut Write) {
        self.memory.save_memory(writer)
    }
    pub fn set_memory(&mut self, address: u16, value: u16) {
        self.memory.set_memory(address,value)
    }
    pub fn get_memory(&self, address: u16) -> u16 {
        self.memory.get_memory(address)
    }
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

