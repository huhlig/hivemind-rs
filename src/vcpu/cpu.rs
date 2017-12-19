/// Modified Implementation of DCPU16
/// https://gist.github.com/metaphox/3888117
///
use std::io::{Read, Write};
use vcpu::memory::Memory;

pub struct HiveCPU {
    memory: Memory,
    sp: u16,
    pc: u16,
    ex: u16,
    ia: u16,
    a: u16,
    b: u16,
    c: u16,
    x: u16,
    y: u16,
    z: u16,
    i: u16,
    j: u16,
}

impl HiveCPU {
    pub fn new() -> HiveCPU {
        HiveCPU {
            memory: Memory::new(),
            sp: u16,
            pc: u16,
            ex: u16,
            ia: u16,
            a: u16,
            b: u16,
            c: u16,
            x: u16,
            y: u16,
            z: u16,
            i: u16,
            j: u16,
        }
    }
    pub fn load_memory(&mut self, reader: &mut Read) {
        self.memory.load_memory(reader)
    }
    pub fn save_memory(&mut self, writer: &mut Write) {
        self.memory.save_memory(writer)
    }
    pub fn set_memory(&mut self, address: u16, value: u16) {
        self.memory.set_memory(address, value)
    }
    pub fn get_memory(&self, address: u16) -> u16 {
        self.memory.get_memory(address)
    }
    pub fn get_sp(&self) -> u16 { self.sp }
    pub fn get_pc(&self) -> u16 { self.pc }
    pub fn get_ex(&self) -> u16 { self.ex }
    pub fn get_ia(&self) -> u16 { self.ia }
    pub fn get_a(&self) -> u16 { self.a }
    pub fn get_b(&self) -> u16 { self.b }
    pub fn get_c(&self) -> u16 { self.c }
    pub fn get_x(&self) -> u16 { self.x }
    pub fn get_y(&self) -> u16 { self.y }
    pub fn get_z(&self) -> u16 { self.z }
    pub fn get_i(&self) -> u16 { self.i }
    pub fn get_j(&self) -> u16 { self.j }
    pub fn step(&mut self) {
        let instruction = self.get_memory(self.pc);
        let opcode =  (instruction & 0x001F);
        let left = (instruction & 0xFC00) >> 10;
        let right = (instruction & 0x03E0) >> 5;
    }
}

