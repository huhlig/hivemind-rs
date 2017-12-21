/// Modified Implementation of DCPU16
/// https://gist.github.com/metaphox/3888117
///
use std::io::{Read, Write};
use std::mem;
use std::slice;

///
/// VCPU State Storage
///
pub struct VCPU16 {
    registers: [u16; 12],
    memory: [u16; 65536],
    state: State,

}

///
/// VCPU Register Index
///
enum Register {
    A = 0x0,
    B = 0x1,
    C = 0x2,
    X = 0x3,
    Y = 0x4,
    Z = 0x5,
    I = 0x6,
    J = 0x7,
    PC = 0x8,
    SP = 0x9,
    EX = 0xA,
    IA = 0xB,
}

///
/// VCPU Operating States
///
enum State {
    Idle,
    Busy(u16, Instruction),
    Sleeping(u16),
    Hibernating,
    Halted,
}

///
/// Decoded Instruction Value
///
enum Value {
    Register { register: Register, value: u16 },
    Memory { address: u16, value: u16 },
    Literal { value: u16 },
    None,
}

struct Decoded<T> {
    pub result: T,
    pub time: usize,
}

enum Instruction {
    ERR,
    NOP,
    HIB,
    JSR { left: Value },
    SLP { left: Value },
    INT { left: Value },
    IAG { left: Value },
    IAS { left: Value },
    RFI { left: Value },
    IAQ { left: Value },
    HWN { left: Value },
    HWQ { left: Value },
    HWI { left: Value },
    SET { left: Value, right: Value },
    ADD { left: Value, right: Value },
    SUB { left: Value, right: Value },
    MUL { left: Value, right: Value },
    MLI { left: Value, right: Value },
    DIV { left: Value, right: Value },
    DVI { left: Value, right: Value },
    MOD { left: Value, right: Value },
    MDI { left: Value, right: Value },
    AND { left: Value, right: Value },
    BOR { left: Value, right: Value },
    XOR { left: Value, right: Value },
    SHR { left: Value, right: Value },
    ASR { left: Value, right: Value },
    SHL { left: Value, right: Value },
    IFB { left: Value, right: Value },
    IFC { left: Value, right: Value },
    IFE { left: Value, right: Value },
    IFN { left: Value, right: Value },
    IFG { left: Value, right: Value },
    IFA { left: Value, right: Value },
    IFL { left: Value, right: Value },
    IFU { left: Value, right: Value },
    ADX { left: Value, right: Value },
    SBX { left: Value, right: Value },
    STI { left: Value, right: Value },
    STD { left: Value, right: Value },
}

impl VCPU16 {
    pub fn new() -> VCPU16 {
        VCPU16 {
            registers: [0; 12],
            memory: [0; 65536],
            state: State::Idle,
        }
    }
    pub fn load_memory(&mut self, reader: &mut Read) {
        unsafe {
            let memory_size = mem::size_of_val(&self.memory);
            let memory_slice = slice::from_raw_parts_mut(
                &mut self.memory as *mut _ as *mut u8,
                memory_size,
            );
            reader.read_exact(memory_slice).unwrap();
        }
    }
    pub fn save_memory(&mut self, writer: &mut Write) {
        unsafe {
            let memory_size = mem::size_of_val(&self.memory);
            let memory_slice = slice::from_raw_parts_mut(
                &mut self.memory as *mut _ as *mut u8,
                memory_size,
            );
            writer.write(memory_slice).unwrap();
        }
    }
    pub fn set_memory(&mut self, address: u16, value: u16) { self.memory[address as usize] = value }
    pub fn get_memory(&self, address: u16) -> u16 { self.memory[address as usize] }
    pub fn get_sp(&self) -> u16 { self.registers[Register::SP as usize] }
    pub fn get_pc(&self) -> u16 { self.registers[Register::PC as usize] }
    pub fn get_ex(&self) -> u16 { self.registers[Register::EX as usize] }
    pub fn get_ia(&self) -> u16 { self.registers[Register::IA as usize] }
    pub fn get_a(&self) -> u16 { self.registers[Register::A as usize] }
    pub fn get_b(&self) -> u16 { self.registers[Register::B as usize] }
    pub fn get_c(&self) -> u16 { self.registers[Register::C as usize] }
    pub fn get_x(&self) -> u16 { self.registers[Register::X as usize] }
    pub fn get_y(&self) -> u16 { self.registers[Register::Y as usize] }
    pub fn get_z(&self) -> u16 { self.registers[Register::Z as usize] }
    pub fn get_i(&self) -> u16 { self.registers[Register::I as usize] }
    pub fn get_j(&self) -> u16 { self.registers[Register::J as usize] }
    ///
    /// Decode Left Value from Instruction Word
    /// LLLLLL----------
    ///
    /// --- Values: (6 bits) --------------------------------------------------------
    ///  C | VALUE     | DESCRIPTION
    /// ---+-----------+----------------------------------------------------------------
    ///  0 | 0x00-0x07 | register (A, B, C, X, Y, Z, I or J, in that order)
    ///  0 | 0x08-0x0f | [register]
    ///  1 | 0x10-0x17 | [register + NEXT]
    ///  0 |      0x18 | (POP / [SP++])
    ///  0 |      0x19 | [SP] / PEEK
    ///  1 |      0x1A | [SP + NEXT] / PICK n
    ///  0 |      0x1B | SP
    ///  0 |      0x1C | PC
    ///  0 |      0x1D | EX
    ///  1 |      0x1E | [NEXT]
    ///  1 |      0x1F | NEXT (literal)
    ///  0 | 0x20-0x3F | literal value 0xffff-0x1e (-1..30) (literal) (only for a)
    /// ---+-----------+----------------------------------------------------------------
    ///
    /// * "NEXT" means "[PC++]". Increases the word length of the instruction by 1.
    /// * By using 0x18, 0x19, 0x1A as PEEK, POP/PUSH, and PICK there's a reverse stack
    ///   starting at memory location 0xFFFF. Example: "SET PUSH, 10", "SET X, POP"
    /// * Attempting to write to a literal value fails silently
    fn decode_left(&mut self, instruction_word: u16) -> Decoded<Value> {
        match (instruction_word & 0xFC00) >> 10 {
            0x00 => { // A
                Decoded {
                    result: Value::Register {
                        register: Register::A,
                        value: self.registers[Register::A as usize],
                    },
                    time: 0,
                }
            }
            0x01 => { // B
                Decoded {
                    result: Value::Register {
                        register: Register::B,
                        value: self.registers[Register::B as usize],
                    },
                    time: 0,
                }
            }
            0x02 => { // C
                Decoded {
                    result: Value::Register {
                        register: Register::C,
                        value: self.registers[Register::C as usize],
                    },
                    time: 0,
                }
            }
            0x03 => { // X
                Decoded {
                    result: Value::Register {
                        register: Register::X,
                        value: self.registers[Register::X as usize],
                    },
                    time: 0,
                }
            }
            0x04 => { // Y
                Decoded {
                    result: Value::Register {
                        register: Register::Y,
                        value: self.registers[Register::Y as usize],
                    },
                    time: 0,
                }
            }
            0x05 => { // Z
                Decoded {
                    result: Value::Register {
                        register: Register::Z,
                        value: self.registers[Register::Z as usize],
                    },
                    time: 0,
                }
            }
            0x06 => { // I
                Decoded {
                    result: Value::Register {
                        register: Register::I,
                        value: self.registers[Register::I as usize],
                    },
                    time: 0,
                }
            }
            0x07 => { // J
                Decoded {
                    result: Value::Register {
                        register: Register::J,
                        value: self.registers[Register::J as usize],
                    },
                    time: 0,
                }
            }
            0x08 => {  // [A]
                let address: u16 = self.registers[Register::A as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x09 => {  // [B]
                let address: u16 = self.registers[Register::A as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x0A => { // [C]
                let address: u16 = self.registers[Register::C as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x0B => { // [X]
                let address: u16 = self.registers[Register::X as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x0C => { // [Y]
                let address: u16 = self.registers[Register::Y as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x0D => { // [Z]
                let address: u16 = self.registers[Register::Z as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x0E => { // [I]
                let address: u16 = self.registers[Register::I as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x0F => { // [J]
                let address: u16 = self.registers[Register::J as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x10 => { // [A + NEXT]
                let base: u16 = self.registers[Register::A as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x11 => { // [B + NEXT]
                let base: u16 = self.registers[Register::B as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x12 => { // [C + NEXT]
                let base: u16 = self.registers[Register::C as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x13 => { // [X + NEXT]
                let base: u16 = self.registers[Register::X as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x14 => { // [Y + NEXT]
                let base: u16 = self.registers[Register::Y as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x15 => { // [Z + NEXT]
                let base: u16 = self.registers[Register::Z as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x16 => { // [I + NEXT]
                let base: u16 = self.registers[Register::I as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x17 => { // [J + NEXT]
                let base: u16 = self.registers[Register::J as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x18 => { // Stack Pop [SP++] (left only)
                let address: u16 = self.registers[Register::SP as usize];
                let value: u16 = self.memory[address as usize];
                self.registers[Register::SP as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x19 => { // Stack Peek [SP]
                let address: u16 = self.registers[Register::SP as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x1A => { // Stack Pick [SP + NEXT]
                let base: u16 = self.registers[Register::SP as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x1B => { // SP
                Decoded {
                    result: Value::Register {
                        register: Register::SP,
                        value: self.registers[Register::SP as usize],
                    },
                    time: 0,
                }
            }
            0x1C => { // PC
                Decoded {
                    result: Value::Register {
                        register: Register::PC,
                        value: self.registers[Register::PC as usize],
                    },
                    time: 0,
                }
            }
            0x1D => { // EX
                Decoded {
                    result: Value::Register {
                        register: Register::EX,
                        value: self.registers[Register::EX as usize],
                    },
                    time: 0,
                }
            }
            0x1E => { // [NEXT]
                let next: u16 = self.registers[Register::PC as usize];
                let address: u16 = self.memory[next as usize];
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x1F => { // NEXT (literal)
                let next: u16 = self.registers[Register::PC as usize];
                let value: u16 = self.memory[next as usize];
                self.registers[Register::PC as usize];
                Decoded { result: Value::Literal { value }, time: 1 }
            }
            0x20 => { Decoded { result: Value::Literal { value: 0xFFFF }, time: 0 } }
            0x21 => { Decoded { result: Value::Literal { value: 0 }, time: 0 } }
            0x22 => { Decoded { result: Value::Literal { value: 1 }, time: 0 } }
            0x23 => { Decoded { result: Value::Literal { value: 2 }, time: 0 } }
            0x24 => { Decoded { result: Value::Literal { value: 3 }, time: 0 } }
            0x25 => { Decoded { result: Value::Literal { value: 4 }, time: 0 } }
            0x26 => { Decoded { result: Value::Literal { value: 5 }, time: 0 } }
            0x27 => { Decoded { result: Value::Literal { value: 6 }, time: 0 } }
            0x28 => { Decoded { result: Value::Literal { value: 7 }, time: 0 } }
            0x29 => { Decoded { result: Value::Literal { value: 8 }, time: 0 } }
            0x2A => { Decoded { result: Value::Literal { value: 9 }, time: 0 } }
            0x2B => { Decoded { result: Value::Literal { value: 10 }, time: 0 } }
            0x2C => { Decoded { result: Value::Literal { value: 11 }, time: 0 } }
            0x2D => { Decoded { result: Value::Literal { value: 12 }, time: 0 } }
            0x2E => { Decoded { result: Value::Literal { value: 13 }, time: 0 } }
            0x2F => { Decoded { result: Value::Literal { value: 14 }, time: 0 } }
            0x30 => { Decoded { result: Value::Literal { value: 15 }, time: 0 } }
            0x31 => { Decoded { result: Value::Literal { value: 16 }, time: 0 } }
            0x32 => { Decoded { result: Value::Literal { value: 17 }, time: 0 } }
            0x33 => { Decoded { result: Value::Literal { value: 18 }, time: 0 } }
            0x34 => { Decoded { result: Value::Literal { value: 19 }, time: 0 } }
            0x35 => { Decoded { result: Value::Literal { value: 20 }, time: 0 } }
            0x36 => { Decoded { result: Value::Literal { value: 21 }, time: 0 } }
            0x37 => { Decoded { result: Value::Literal { value: 22 }, time: 0 } }
            0x38 => { Decoded { result: Value::Literal { value: 23 }, time: 0 } }
            0x39 => { Decoded { result: Value::Literal { value: 24 }, time: 0 } }
            0x3A => { Decoded { result: Value::Literal { value: 25 }, time: 0 } }
            0x3B => { Decoded { result: Value::Literal { value: 26 }, time: 0 } }
            0x3C => { Decoded { result: Value::Literal { value: 27 }, time: 0 } }
            0x3D => { Decoded { result: Value::Literal { value: 28 }, time: 0 } }
            0x3E => { Decoded { result: Value::Literal { value: 29 }, time: 0 } }
            0x3F => { Decoded { result: Value::Literal { value: 30 }, time: 0 } }
            _ => { Decoded { result: Value::None, time: 0 } }
        }
    }
    ///
    /// Decode Right Value from Instruction Word
    /// ------RRRRR-----
    /// --- Values: (5 bits) --------------------------------------------------------
    ///  C | VALUE     | DESCRIPTION
    /// ---+-----------+----------------------------------------------------------------
    ///  0 | 0x00-0x07 | register (A, B, C, X, Y, Z, I or J, in that order)
    ///  0 | 0x08-0x0f | [register]
    ///  1 | 0x10-0x17 | [register + NEXT]
    ///  0 |      0x18 | (PUSH / [--SP])
    ///  0 |      0x19 | [SP] / PEEK
    ///  1 |      0x1a | [SP + NEXT] / PICK next
    ///  0 |      0x1b | SP
    ///  0 |      0x1c | PC
    ///  0 |      0x1d | EX
    ///  1 |      0x1e | [NEXT]
    ///  1 |      0x1f | NEXT (literal)
    /// ---+-----------+----------------------------------------------------------------
    ///
    /// * "NEXT" means "[PC++]". Increases the word length of the instruction by 1.
    /// * By using 0x18, 0x19, 0x1a as PEEK, POP/PUSH, and PICK there's a reverse stack
    ///   starting at memory location 0xffff. Example: "SET PUSH, 10", "SET X, POP"
    /// * Attempting to write to a literal value fails silently
    fn decode_right(&mut self, instruction_word: u16) -> Decoded<Value> {
        match (instruction_word & 0x03E0) >> 5 {
            0x00 => { // A
                Decoded {
                    result: Value::Register {
                        register: Register::A,
                        value: self.registers[Register::A as usize],
                    },
                    time: 0,
                }
            }
            0x01 => { // B
                Decoded {
                    result: Value::Register {
                        register: Register::B,
                        value: self.registers[Register::B as usize],
                    },
                    time: 0,
                }
            }
            0x02 => { // C
                Decoded {
                    result: Value::Register {
                        register: Register::C,
                        value: self.registers[Register::C as usize],
                    },
                    time: 0,
                }
            }
            0x03 => { // X
                Decoded {
                    result: Value::Register {
                        register: Register::X,
                        value: self.registers[Register::X as usize],
                    },
                    time: 0,
                }
            }
            0x04 => { // Y
                Decoded {
                    result: Value::Register {
                        register: Register::Y,
                        value: self.registers[Register::Y as usize],
                    },
                    time: 0,
                }
            }
            0x05 => { // Z
                Decoded {
                    result: Value::Register {
                        register: Register::Z,
                        value: self.registers[Register::Z as usize],
                    },
                    time: 0,
                }
            }
            0x06 => { // I
                Decoded {
                    result: Value::Register {
                        register: Register::I,
                        value: self.registers[Register::I as usize],
                    },
                    time: 0,
                }
            }
            0x07 => { // J
                Decoded {
                    result: Value::Register {
                        register: Register::J,
                        value: self.registers[Register::J as usize],
                    },
                    time: 0,
                }
            }
            0x08 => {  // [A]
                let address: u16 = self.registers[Register::A as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x09 => {  // [B]
                let address: u16 = self.registers[Register::A as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x0A => { // [C]
                let address: u16 = self.registers[Register::C as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x0B => { // [X]
                let address: u16 = self.registers[Register::X as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x0C => { // [Y]
                let address: u16 = self.registers[Register::Y as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x0D => { // [Z]
                let address: u16 = self.registers[Register::Z as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x0E => { // [I]
                let address: u16 = self.registers[Register::I as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x0F => { // [J]
                let address: u16 = self.registers[Register::J as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x10 => { // [A + NEXT]
                let base: u16 = self.registers[Register::A as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x11 => { // [B + NEXT]
                let base: u16 = self.registers[Register::B as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x12 => { // [C + NEXT]
                let base: u16 = self.registers[Register::C as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x13 => { // [X + NEXT]
                let base: u16 = self.registers[Register::X as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x14 => { // [Y + NEXT]
                let base: u16 = self.registers[Register::Y as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x15 => { // [Z + NEXT]
                let base: u16 = self.registers[Register::Z as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x16 => { // [I + NEXT]
                let base: u16 = self.registers[Register::I as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x17 => { // [J + NEXT]
                let base: u16 = self.registers[Register::J as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x18 => { // Stack Push [--SP] (right only)
                self.registers[Register::SP as usize] -= 1;
                let address: u16 = self.registers[Register::SP as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x19 => { // Stack Peek [SP]
                let address: u16 = self.registers[Register::SP as usize];
                let value: u16 = self.memory[address as usize];
                Decoded { result: Value::Memory { address, value }, time: 0 }
            }
            0x1A => { // Stack Pick [SP + NEXT]
                let base: u16 = self.registers[Register::SP as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x1B => { // SP
                Decoded {
                    result: Value::Register {
                        register: Register::SP,
                        value: self.registers[Register::SP as usize],
                    },
                    time: 0,
                }
            }
            0x1C => { // PC
                Decoded {
                    result: Value::Register {
                        register: Register::PC,
                        value: self.registers[Register::PC as usize],
                    },
                    time: 0,
                }
            }
            0x1D => { // EX
                Decoded {
                    result: Value::Register {
                        register: Register::EX,
                        value: self.registers[Register::EX as usize],
                    },
                    time: 0,
                }
            }
            0x1E => { // [NEXT]
                let next: u16 = self.registers[Register::PC as usize];
                let address: u16 = self.memory[next as usize];
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                Decoded { result: Value::Memory { address, value }, time: 1 }
            }
            0x1F => { // NEXT (literal)
                let next: u16 = self.registers[Register::PC as usize];
                let value: u16 = self.memory[next as usize];
                self.registers[Register::PC as usize];
                Decoded { result: Value::Literal { value }, time: 1 }
            }
            _ => Decoded { result: Value::None, time: 0 }
        }
    }
    /// Nullary opcodes always have their lower ten bits unset, have no values and a
    /// six bit opcode. In binary, they have the format: oooooo0000000000
    /// --- Magical opcodes: (5 bits) --------------------------------------------------
    ///  C | VAL  | NAME  | DESCRIPTION
    /// ---+------+-------+-------------------------------------------------------------
    ///  - | 0x00 | NOP   | No Operation
    ///  * | 0x01 | HIB   | Hibernate until Interrupted
    ///  - | 0x02 | -     | Unused
    ///  - | 0x03 | -     | Unused
    ///  - | 0x04 | -     | Unused
    ///  - | 0x05 | -     | Unused
    ///  - | 0x06 | -     | Unused
    ///  - | 0x07 | -     | Unused
    ///  - | 0x08 | -     | Unused
    ///  - | 0x09 | -     | Unused
    ///  - | 0x0A | -     | Unused
    ///  - | 0x0B | -     | Unused
    ///  - | 0x0C | -     | Unused
    ///  - | 0x0D | -     | Unused
    ///  - | 0x0E | -     | Unused
    ///  - | 0x0F | -     | Unused
    ///  - | 0x10 | -     | Unused
    ///  - | 0x11 | -     | Unused
    ///  - | 0x12 | -     | Unused
    ///  - | 0x13 | -     | Unused
    ///  - | 0x14 | -     | Unused
    ///  - | 0x15 | -     | Unused
    ///  - | 0x16 | -     | Unused
    ///  - | 0x17 | -     | Unused
    ///  - | 0x18 | -     | Unused
    ///  - | 0x19 | -     | Unused
    ///  - | 0x1A | -     | Unused
    ///  - | 0x1B | -     | Unused
    ///  - | 0x1C | -     | Unused
    ///  - | 0x1D | -     | Unused
    ///  - | 0x1E | -     | Unused
    ///  - | 0x1F | -     | Unused
    ///  - | 0x20 | -     | Unused
    ///  - | 0x21 | -     | Unused
    ///  - | 0x22 | -     | Unused
    ///  - | 0x23 | -     | Unused
    ///  - | 0x24 | -     | Unused
    ///  - | 0x25 | -     | Unused
    ///  - | 0x26 | -     | Unused
    ///  - | 0x27 | -     | Unused
    ///  - | 0x28 | -     | Unused
    ///  - | 0x29 | -     | Unused
    ///  - | 0x2A | -     | Unused
    ///  - | 0x2B | -     | Unused
    ///  - | 0x2C | -     | Unused
    ///  - | 0x2D | -     | Unused
    ///  - | 0x2E | -     | Unused
    ///  - | 0x2F | -     | Unused
    ///  - | 0x30 | -     | Unused
    ///  - | 0x31 | -     | Unused
    ///  - | 0x32 | -     | Unused
    ///  - | 0x33 | -     | Unused
    ///  - | 0x34 | -     | Unused
    ///  - | 0x35 | -     | Unused
    ///  - | 0x36 | -     | Unused
    ///  - | 0x37 | -     | Unused
    ///  - | 0x38 | -     | Unused
    ///  - | 0x39 | -     | Unused
    ///  - | 0x3A | -     | Unused
    ///  - | 0x3B | -     | Unused
    ///  - | 0x3C | -     | Unused
    ///  - | 0x3D | -     | Unused
    ///  - | 0x3E | -     | Unused
    ///  - | 0x3F | -     | Unused
    /// ---+------+-------+-------------------------------------------------------------
    fn decode_nullary(&mut self, instruction_word: u16) -> Decoded<Instruction> {
        match (instruction_word & 0xFC00) >> 10 {
            0x00 => Decoded { result: Instruction::NOP, time: 0 },
            0x01 => Decoded { result: Instruction::HIB, time: 0 },
            _ => Decoded { result: Instruction::ERR, time: 0 },
        }
    }
    ///
    /// Decode Unary Instruction
    /// Unary opcodes always have their lower five bits unset, have one value and a
    /// five bit opcode. In binary, they have the format: aaaaaaooooo00000
    /// The value (L) is in the same six bit format as defined earlier.
    ///
    /// --- Special opcodes: (5 bits) --------------------------------------------------
    ///  C | VAL  | NAME  | DESCRIPTION
    /// ---+------+-------+-------------------------------------------------------------
    ///  - | 0x00 | n/a   | Reserved for future expansion
    ///  3 | 0x01 | JSR L | pushes the address of the next instruction to the stack,
    ///    |      |       | then sets PC to L
    ///  - | 0x02 | SLP L | Sleep a cycles
    ///  - | 0x03 | -     | Unused
    ///  - | 0x04 | -     | Unused
    ///  - | 0x05 | -     | Unused
    ///  - | 0x06 | -     | Unused
    ///  - | 0x07 | -     | Unused
    ///  4 | 0x08 | INT L | triggers a software interrupt with message a
    ///  1 | 0x09 | IAG L | sets L to IA
    ///  1 | 0x0A | IAS L | sets IA to L
    ///  3 | 0x0B | RFI L | disables interrupt queueing, pops A from the stack, then
    ///    |      |       | pops PC from the stack
    ///  2 | 0x0C | IAQ L | if L is nonzero, interrupts will be added to the queue
    ///    |      |       | instead of triggered. if L is zero, interrupts will be
    ///    |      |       | triggered as normal again
    ///  - | 0x0D | -     | Unused
    ///  - | 0x0E | -     | Unused
    ///  - | 0x0F | -     | Unused
    ///  2 | 0x10 | HWN L | sets a to number of connected hardware devices
    ///  4 | 0x11 | HWQ L | sets A, B, C, X, Y registers to information about hardware L
    ///    |      |       | A+(B<<16) is a 32 bit word identifying the hardware id
    ///    |      |       | C is the hardware version
    ///    |      |       | X+(Y<<16) is a 32 bit word identifying the manufacturer
    ///  4+| 0x12 | HWI L | sends an interrupt to hardware L
    ///  - | 0x13 | -     | Unused
    ///  - | 0x14 | -     | Unused
    ///  - | 0x15 | -     | Unused
    ///  - | 0x16 | -     | Unused
    ///  - | 0x17 | -     | Unused
    ///  - | 0x18 | -     | Unused
    ///  - | 0x19 | -     | Unused
    ///  - | 0x1A | -     | Unused
    ///  - | 0x1B | -     | Unused
    ///  - | 0x1C | -     | Unused
    ///  - | 0x1D | -     | Unused
    ///  - | 0x1E | -     | Unused
    ///  - | 0x1F | -     | Unused
    /// ---+------+-------+-------------------------------------------------------------
    fn decode_unary(&mut self, instruction_word: u16) -> Decoded<Instruction> {
        let (left, ltime) = {
            let value = self.decode_left(instruction_word);
            (value.result, value.time)
        };
        match (instruction_word & 0x03E0) >> 5 {
            0x01 => Decoded { result: Instruction::JSR { left }, time: 3 + ltime },
            0x08 => Decoded { result: Instruction::INT { left }, time: 4 + ltime },
            0x09 => Decoded { result: Instruction::IAG { left }, time: 1 + ltime },
            0x0A => Decoded { result: Instruction::IAS { left }, time: 1 + ltime },
            0x0B => Decoded { result: Instruction::RFI { left }, time: 3 + ltime },
            0x0C => Decoded { result: Instruction::IAQ { left }, time: 2 + ltime },
            0x10 => Decoded { result: Instruction::HWN { left }, time: 2 + ltime },
            0x11 => Decoded { result: Instruction::HWQ { left }, time: 4 + ltime },
            0x12 => Decoded { result: Instruction::HWI { left }, time: 4 + ltime },
            _ => Decoded { result: Instruction::ERR, time: 0 },
        }
    }
    ///
    /// Decode Binary Instruction
    /// Decode Instruction
    /// --- Binary opcodes (5 bits) ----------------------------------------------------
    ///  C | VAL  | NAME     | DESCRIPTION
    /// ---+------+----------+----------------------------------------------------------
    ///  - | 0x00 | n/a      | Special Instruction
    ///  1 | 0x01 | SET b, a | sets b to a
    ///  2 | 0x02 | ADD b, a | sets b to b+a, sets EX to 0x0001 if there's an overflow,
    ///    |      |          | 0x0 otherwise
    ///  2 | 0x03 | SUB b, a | sets b to b-a, sets EX to 0xffff if there's an underflow,
    ///    |      |          | 0x0 otherwise
    ///  2 | 0x04 | MUL b, a | sets b to b*a, sets EX to ((b*a)>>16)&0xffff (treats b,
    ///  |      |          | a as unsigned)
    ///  2 | 0x05 | MLI b, a | like MUL, but treat b, a as signed
    ///  3 | 0x06 | DIV b, a | sets b to b/a, sets EX to ((b<<16)/a)&0xffff. if a==0,
    ///    |      |          | sets b and EX to 0 instead. (treats b, a as unsigned)
    ///  3 | 0x07 | DVI b, a | like DIV, but treat b, a as signed. Rounds towards 0
    ///  3 | 0x08 | MOD b, a | sets b to b%a. if a==0, sets b to 0 instead.
    ///  3 | 0x09 | MDI b, a | like MOD, but treat b, a as signed. (MDI -7, 16 == -7)
    ///  1 | 0x0A | AND b, a | sets b to b&a
    ///  1 | 0x0B | BOR b, a | sets b to b|a
    ///  1 | 0x0C | XOR b, a | sets b to b^a
    ///  1 | 0x0D | SHR b, a | sets b to b>>>a, sets EX to ((b<<16)>>a)&0xffff
    ///    |      |          | (logical shift)
    ///  1 | 0x0E | ASR b, a | sets b to b>>a, sets EX to ((b<<16)>>>a)&0xffff
    ///    |      |          | (arithmetic shift) (treats b as signed)
    ///  1 | 0x0F | SHL b, a | sets b to b<<a, sets EX to ((b<<a)>>16)&0xffff
    ///  2+| 0x10 | IFB b, a | performs next instruction only if (b&a)!=0
    ///  2+| 0x11 | IFC b, a | performs next instruction only if (b&a)==0
    ///  2+| 0x12 | IFE b, a | performs next instruction only if b==a
    ///  2+| 0x13 | IFN b, a | performs next instruction only if b!=a
    ///  2+| 0x14 | IFG b, a | performs next instruction only if b>a
    ///  2+| 0x15 | IFA b, a | performs next instruction only if b>a (signed)
    ///  2+| 0x16 | IFL b, a | performs next instruction only if b<a
    ///  2+| 0x17 | IFU b, a | performs next instruction only if b<a (signed)
    ///  - | 0x18 | -        | Unused
    ///  - | 0x19 | -        | Unused
    ///  3 | 0x1A | ADX b, a | sets b to b+a+EX, sets EX to 0x0001 if there is an overflow,
    ///    |      |          | 0x0 otherwise
    ///  3 | 0x1B | SBX b, a | sets b to b-a+EX, sets EX to 0xFFFF if there is an underflow,
    ///    |      |          | 0x0 otherwise
    ///  - | 0x1C | -        | Unused
    ///  - | 0x1D | -        | Unused
    ///  2 | 0x1E | STI b, a | sets b to a, then increases I and J by 1
    ///  2 | 0x1F | STD b, a | sets b to a, then decreases I and J by 1
    /// ---+------+----------+----------------------------------------------------------
    ///
    ///  * The branching opcodes take one cycle longer to perform if the test fails
    ///    When they skip an if instruction, they will skip an additional instruction
    ///    at the cost of one extra cycle. This lets you easily chain conditionals.
    ///  * Signed numbers are represented using two's complement.
    fn decode_binary(&mut self, instruction_word: u16) -> Decoded<Instruction> {
        let (left, ltime) = {
            let value = self.decode_left(instruction_word);
            (value.result, value.time)
        };
        let (right, rtime) = {
            let value = self.decode_left(instruction_word);
            (value.result, value.time)
        };
        let time = ltime + rtime;
        match (instruction_word & 0x001F) >> 0 {
            0x01 => Decoded { result: Instruction::SET { left, right }, time: 0 + time },
            0x02 => Decoded { result: Instruction::ADD { left, right }, time: 2 + time },
            0x03 => Decoded { result: Instruction::SUB { left, right }, time: 2 + time },
            0x04 => Decoded { result: Instruction::MUL { left, right }, time: 2 + time },
            0x05 => Decoded { result: Instruction::MLI { left, right }, time: 2 + time },
            0x06 => Decoded { result: Instruction::DIV { left, right }, time: 3 + time },
            0x07 => Decoded { result: Instruction::DVI { left, right }, time: 3 + time },
            0x08 => Decoded { result: Instruction::MOD { left, right }, time: 3 + time },
            0x09 => Decoded { result: Instruction::MDI { left, right }, time: 3 + time },
            0x0A => Decoded { result: Instruction::AND { left, right }, time: 1 + time },
            0x0B => Decoded { result: Instruction::BOR { left, right }, time: 1 + time },
            0x0C => Decoded { result: Instruction::XOR { left, right }, time: 1 + time },
            0x0D => Decoded { result: Instruction::SHR { left, right }, time: 1 + time },
            0x0E => Decoded { result: Instruction::ASR { left, right }, time: 1 + time },
            0x0F => Decoded { result: Instruction::SHL { left, right }, time: 1 + time },
            0x10 => Decoded { result: Instruction::IFB { left, right }, time: 2 + time },
            0x11 => Decoded { result: Instruction::IFC { left, right }, time: 2 + time },
            0x12 => Decoded { result: Instruction::IFE { left, right }, time: 2 + time },
            0x13 => Decoded { result: Instruction::IFN { left, right }, time: 2 + time },
            0x14 => Decoded { result: Instruction::IFG { left, right }, time: 2 + time },
            0x15 => Decoded { result: Instruction::IFA { left, right }, time: 2 + time },
            0x16 => Decoded { result: Instruction::IFL { left, right }, time: 2 + time },
            0x17 => Decoded { result: Instruction::IFU { left, right }, time: 2 + time },
            0x18 => Decoded { result: Instruction::ERR, time: 1 },
            0x19 => Decoded { result: Instruction::ERR, time: 1 },
            0x1A => Decoded { result: Instruction::ADX { left, right }, time: 3 + time },
            0x1B => Decoded { result: Instruction::SBX { left, right }, time: 3 + time },
            0x1C => Decoded { result: Instruction::ERR, time: 1 },
            0x1D => Decoded { result: Instruction::ERR, time: 1 },
            0x1E => Decoded { result: Instruction::STI { left, right }, time: 2 + time },
            0x1F => Decoded { result: Instruction::STD { left, right }, time: 2 + time },
            _ => Decoded { result: Instruction::ERR, time: 0 }
        }
    }

    ///
    /// Decode Next Instruction
    ///
    fn decode(&mut self) -> Decoded<Instruction> {
        let address: u16 = self.registers[Register::PC as usize];
        let instruction_word: u16 = self.memory[address as usize];
        self.registers[Register::PC as usize] += 1;
        if instruction_word & 0x03FF == 0 {
            self.decode_nullary(instruction_word)
        } else if instruction_word & 0x001F == 0 {
            self.decode_unary(instruction_word)
        } else {
            self.decode_binary(instruction_word)
        }
    }

    /// Execute Instruction
    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            _ => {
                //TODO: Stop doing nothing
            }
        }
    }

    pub fn step(&mut self) {
        match &self.state {
            &State::Idle => {
                let (ref instruction, cycles) = {
                    let instruction = self.decode();
                    (instruction.result, instruction.time)
                };

                self.execute(&instruction);
            }
            &State::Busy(time, instruction) => {}
            &State::Sleeping(time) => {
                self.state = State::Sleeping(time - 1);
            }
            &State::Hibernating => {
                // Wake up on Interrupt
            }
            &State::Halted => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VCPU16;
    use rand::{Rng, SeedableRng, XorShiftRng};
    use std::io::Cursor;

    #[test]
    pub fn test_save_load_memory() {
        // Create our Memory and external buffers
        let mut output: [u8; 131072] = [0; 131072];
        let mut input: [u8; 131072] = [0; 131072];
        let mut vcpu = VCPU16::new();

        // Fill our input Buffer
        XorShiftRng::from_seed([1; 4]).fill_bytes(&mut input[..]);

        // Load our input into Memory
        vcpu.load_memory(&mut Cursor::new(&mut input[..]));

        // Save our memory to output
        vcpu.save_memory(&mut Cursor::new(&mut output[..]));

        // Compare buffers
        assert_eq!(&input[..], &output[..]);
    }
}