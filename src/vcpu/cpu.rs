/// Modified Implementation of DCPU16
/// https://gist.github.com/metaphox/3888117
///
use std::io::{Read, Write};
use std::mem;
use std::slice;

pub struct VCPU16 {
    registers: [u16; 12],
    memory: [u16; 65536],
    state: State,
    busy: u16,
}

pub enum State {
    Hibernating,
    Sleeping,
    Active,
    Halted,
    Busy,
}

pub enum Register {
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

pub enum Value {
    Register { register: Register, value: u16 },
    Memory { address: u16, value: u16 },
    Literal { value: u16 },
}

pub enum Instruction {
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
            state: State::Active,
            busy: 0,
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
    fn decode_left(&mut self, instruction_word: u16) -> Value {
        match (instruction_word & 0xFC00) >> 10 {
            0x00 => { // A
                Value::Register {
                    register: Register::A,
                    value: self.registers[Register::A as usize],
                }
            }
            0x01 => { // B
                Value::Register {
                    register: Register::B,
                    value: self.registers[Register::B as usize],
                }
            }
            0x02 => { // C
                Value::Register {
                    register: Register::C,
                    value: self.registers[Register::C as usize],
                }
            }
            0x03 => { // X
                Value::Register {
                    register: Register::X,
                    value: self.registers[Register::X as usize],
                }
            }
            0x04 => { // Y
                Value::Register {
                    register: Register::Y,
                    value: self.registers[Register::Y as usize],
                }
            }
            0x05 => { // Z
                Value::Register {
                    register: Register::Z,
                    value: self.registers[Register::Z as usize],
                }
            }
            0x06 => { // I
                Value::Register {
                    register: Register::I,
                    value: self.registers[Register::I as usize],
                }
            }
            0x07 => { // J
                Value::Register {
                    register: Register::J,
                    value: self.registers[Register::J as usize],
                }
            }
            0x08 => {  // [A]
                let address: u16 = self.registers[Register::A as usize];
                let value: u16 = self.memory[address as usize];
                Value::Memory { address, value }
            }
            0x09 => {  // [B]
                let address: u16 = self.registers[Register::B as usize];
                let value: u16 = self.memory[address as usize];
                Value::Memory { address, value }
            }
            // TODO: Start Here
            0x0A => Value::Memory { // [C]
                address: self.registers[Register::C as usize],
                value: self.memory[self.registers[Register::C as usize] as usize],
            },
            0x0B => Value::Memory { // [X]
                address: self.registers[Register::X as usize],
                value: self.memory[self.registers[Register::X as usize] as usize],
            },
            0x0C => Value::Memory { // [Y]
                address: self.registers[Register::Y as usize],
                value: self.memory[self.registers[Register::Y as usize] as usize],
            },
            0x0D => Value::Memory {
                address: self.registers[Register::Z as usize],
                value: self.memory[self.registers[Register::Z as usize] as usize],
            }, // [z]
            0x0E => Value::Memory { address: self.registers[Register::I as usize], value: self.memory[self.registers[Register::I as usize] as usize] }, // [i]
            0x0F => Value::Memory { address: self.registers[Register::J as usize], value: self.memory[self.registers[Register::J as usize] as usize] }, // [j]
            0x10 => { // [A + NEXT]
                let base: u16 = self.registers[Register::A as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x11 => { // [B + NEXT]
                let base: u16 = self.registers[Register::B as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x12 => { // [C + NEXT]
                let base: u16 = self.registers[Register::C as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC as usize] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x13 => { // [X + NEXT]
                let base: u16 = self.registers[Register::X as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x14 => { // [Y + NEXT]
                let base: u16 = self.registers[Register::Y as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x15 => { // [Z + NEXT]
                let base: u16 = self.registers[Register::Z as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x16 => { // [I + NEXT]
                let base: u16 = self.registers[Register::I as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x17 => { // [J + NEXT]
                let base: u16 = self.registers[Register::J as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x18 => { // Stack Pop [SP++] (left only)
                let address: u16 = self.registers[Register::SP as usize];
                let value: u16 = self.memory[address as usize];
                self.registers[Register::SP] += 1;
                Value::Memory { address, value }
            }
            0x19 => { // Stack Peek [SP]
                let address: u16 = self.registers[Register::SP];
                let value: u16 = self.memory[address as usize];
                Value::Memory { address, value }
            }
            0x1A => { // Stack Pick [SP + NEXT]
                let base: u16 = self.registers[Register::SP as usize];
                let next: u16 = self.registers[Register::PC as usize];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x1B => { Value::Register { register: Register::SP, value: self.registers[Register::SP as usize] } } // sp
            0x1C => { Value::Register { register: Register::PC, value: self.registers[Register::PC as usize] } } // pc
            0x1D => { Value::Register { register: Register::EX, value: self.registers[Register::EX as usize] } } // sp
            0x1E => { // [NEXT]
                let next: u16 = self.registers[Register::PC];
                let address: u16 = self.memory[next as usize];
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x1F => { // NEXT (literal)
                let next: u16 = self.registers[Register::PC];
                let value: u16 = self.memory[next as usize];
                self.registers[Register::PC];
                self.delay += 1;
                Value::Literal { value }
            }
            0x20 => { Value::Literal { value: -1 } }
            0x21 => { Value::Literal { value: 0 } }
            0x22 => { Value::Literal { value: 1 } }
            0x23 => { Value::Literal { value: 2 } }
            0x24 => { Value::Literal { value: 3 } }
            0x25 => { Value::Literal { value: 4 } }
            0x26 => { Value::Literal { value: 5 } }
            0x27 => { Value::Literal { value: 6 } }
            0x28 => { Value::Literal { value: 7 } }
            0x29 => { Value::Literal { value: 8 } }
            0x2A => { Value::Literal { value: 9 } }
            0x2B => { Value::Literal { value: 10 } }
            0x2C => { Value::Literal { value: 11 } }
            0x2D => { Value::Literal { value: 12 } }
            0x2E => { Value::Literal { value: 13 } }
            0x2F => { Value::Literal { value: 14 } }
            0x30 => { Value::Literal { value: 15 } }
            0x31 => { Value::Literal { value: 16 } }
            0x32 => { Value::Literal { value: 17 } }
            0x33 => { Value::Literal { value: 18 } }
            0x34 => { Value::Literal { value: 19 } }
            0x35 => { Value::Literal { value: 20 } }
            0x36 => { Value::Literal { value: 21 } }
            0x37 => { Value::Literal { value: 22 } }
            0x38 => { Value::Literal { value: 23 } }
            0x39 => { Value::Literal { value: 24 } }
            0x3A => { Value::Literal { value: 25 } }
            0x3B => { Value::Literal { value: 26 } }
            0x3C => { Value::Literal { value: 27 } }
            0x3D => { Value::Literal { value: 28 } }
            0x3E => { Value::Literal { value: 29 } }
            0x3F => { Value::Literal { value: 30 } }
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
    fn decode_right(&mut self, instruction_word: u16) -> Value {
        match (instruction_word & 0x03E0) >> 5 {
            0x00 => Value::Register { register: Register::A, value: self.a }, // a
            0x01 => Value::Register { register: Register::B, value: self.b }, // b
            0x02 => Value::Register { register: Register::C, value: self.c }, // c
            0x03 => Value::Register { register: Register::X, value: self.x }, // x
            0x04 => Value::Register { register: Register::Y, value: self.y }, // y
            0x05 => Value::Register { register: Register::Z, value: self.z }, // z
            0x06 => Value::Register { register: Register::I, value: self.i }, // i
            0x07 => Value::Register { register: Register::J, value: self.j }, // j
            0x08 => Value::Memory { address: self.a, value: self.memory[self.a as usize] }, // [a]
            0x09 => Value::Memory { address: self.b, value: self.memory[self.b as usize] }, // [b]
            0x0A => Value::Memory { address: self.c, value: self.memory[self.c as usize] }, // [c]
            0x0B => Value::Memory { address: self.x, value: self.memory[self.x as usize] }, // [x]
            0x0C => Value::Memory { address: self.y, value: self.memory[self.y as usize] }, // [y]
            0x0D => Value::Memory { address: self.z, value: self.memory[self.z as usize] }, // [z]
            0x0E => Value::Memory { address: self.i, value: self.memory[self.i as usize] }, // [i]
            0x0F => Value::Memory { address: self.j, value: self.memory[self.j as usize] }, // [j]
            0x10 => { // [A + NEXT]
                let base: u16 = self.registers[Register::A];
                let next: u16 = self.registers[Register::PC];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x11 => { // [B + NEXT]
                let base: u16 = self.registers[Register::B];
                let next: u16 = self.registers[Register::PC];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x12 => { // [C + NEXT]
                let base: u16 = self.registers[Register::C];
                let next: u16 = self.registers[Register::PC];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x13 => { // [X + NEXT]
                let base: u16 = self.registers[Register::X];
                let next: u16 = self.registers[Register::PC];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x14 => { // [Y + NEXT]
                let base: u16 = self.registers[Register::Y];
                let next: u16 = self.registers[Register::PC];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x15 => { // [Z + NEXT]
                let base: u16 = self.registers[Register::Z];
                let next: u16 = self.registers[Register::PC];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x16 => { // [I + NEXT]
                let base: u16 = self.registers[Register::I];
                let next: u16 = self.registers[Register::PC];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x17 => { // [J + NEXT]
                let base: u16 = self.registers[Register::J];
                let next: u16 = self.registers[Register::PC];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x18 => { // Stack Push [--SP] (right only)
                self.registers[Register::SP] -= 1;
                let address: u16 = self.registers[Register::SP];
                let value: u16 = self.memory[address];
                Value::Memory { address, value }
            }
            0x19 => { // Stack Peek [SP]
                let address: u16 = self.registers[Register::SP];
                let value: u16 = self.memory[self.sp as usize];
                Value::Memory { address, value }
            }
            0x1A => { // Stack Pick [SP + NEXT]
                let base: u16 = self.registers[Register::SP];
                let next: u16 = self.registers[Register::PC];
                let offset: u16 = self.memory[next as usize];
                let address: u16 = base + offset;
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x1B => { Value::Register { register: Register::SP, value: self.registers[Register::SP] } } // sp
            0x1C => { Value::Register { register: Register::PC, value: self.registers[Register::PC] } } // pc
            0x1D => { Value::Register { register: Register::EX, value: self.registers[Register::EX] } } // sp
            0x1E => { // [NEXT]
                let next: u16 = self.registers[Register::PC];
                let address: u16 = self.memory[next as usize];
                let value: u16 = self.memory[address as usize];
                self.registers[Register::PC] += 1;
                self.delay += 1;
                Value::Memory { address, value }
            }
            0x1F => { // NEXT (literal)
                let next: u16 = self.registers[Register::PC];
                let value: u16 = self.memory[next as usize];
                self.registers[Register::PC];
                self.delay += 1;
                Value::Literal { value }
            }
        }
    }
    /// Magical opcodes always have their lower ten bits unset, have no values and a
    /// six bit opcode. In binary, they have the format: oooooo----------
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
    fn decode_nullary(&mut self, instruction_word: u16) -> Instruction {
        match (instruction_word & 0xFC00) >> 10 {
            0x00 => Instruction::NOP,
            0x01 => Instruction::HIB,
            _ => Instruction::ERR,
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
    fn decode_unary(&mut self, instruction_word: u16) -> Instruction {
        let left = self.decode_left(instruction_word);
        match (instruction_word & 0x03E0) >> 5 {
            0x01 => { Instruction::JSR { left } }
            0x08 => { Instruction::INT { left } }
            0x09 => { Instruction::IAG { left } }
            0x0A => { Instruction::IAS { left } }
            0x0B => { Instruction::RFI { left } }
            0x0C => { Instruction::IAQ { left } }
            0x10 => { Instruction::HWN { left } }
            0x11 => { Instruction::HWQ { left } }
            0x12 => { Instruction::HWI { left } }
            _ => Instruction::ERR,
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
    fn decode_binary(&mut self, instruction_word: u16) -> Instruction {
        let left = self.decode_left(instruction_word);
        let right = self.decode_right(instruction_word);
        match (instruction_word & 0x001F) >> 0 {
            0x01 => { Instruction::SET { left, right } }
            0x02 => { Instruction::ADD { left, right } }
            0x03 => { Instruction::SUB { left, right } }
            0x04 => { Instruction::MUL { left, right } }
            0x05 => { Instruction::MLI { left, right } }
            0x06 => { Instruction::DIV { left, right } }
            0x07 => { Instruction::DVI { left, right } }
            0x08 => { Instruction::MOD { left, right } }
            0x09 => { Instruction::MDI { left, right } }
            0x0A => { Instruction::AND { left, right } }
            0x0B => { Instruction::BOR { left, right } }
            0x0C => { Instruction::XOR { left, right } }
            0x0D => { Instruction::SHR { left, right } }
            0x0E => { Instruction::ASR { left, right } }
            0x0F => { Instruction::SHL { left, right } }
            0x10 => { Instruction::IFB { left, right } }
            0x11 => { Instruction::IFC { left, right } }
            0x12 => { Instruction::IFE { left, right } }
            0x13 => { Instruction::IFN { left, right } }
            0x14 => { Instruction::IFG { left, right } }
            0x15 => { Instruction::IFA { left, right } }
            0x16 => { Instruction::IFL { left, right } }
            0x17 => { Instruction::IFU { left, right } }
            0x18 => { Instruction::ERR }
            0x19 => { Instruction::ERR }
            0x1A => { Instruction::ADX { left, right } }
            0x1B => { Instruction::SBX { left, right } }
            0x1C => { Instruction::ERR }
            0x1D => { Instruction::ERR }
            0x1E => { Instruction::STI { left, right } }
            0x1F => { Instruction::STD { left, right } }
        }
    }

    ///
    /// Decode Next Instruction
    ///
    fn decode(&mut self) {
        let instruction_word: u16 = self.data[self.registers[Register::PC]];
        self.registers[Register::PC] += 1;
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
        match instruction {}
    }

    pub fn step(&mut self) {
        self.execute(self.decode());
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