
use opcodes::{Opcode, Operand};
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Register {
    A = 0,
    B,
    C,
    X,
    Y,
    Z,
    I,
    J
}

#[derive(Debug)]
pub struct VirtualMachine {
    registers: Box<[i32]>,
    stack: Box<[u16]>,
    ram: Box<[u16]>,
    pc: i32,
    sp: i32,
    ia: i32,
    ex: i32
}

impl VirtualMachine {
    pub fn new() -> VirtualMachine {
        VirtualMachine{
            registers: vec![0i32; 8].into_boxed_slice(),
            stack: vec![0u16; 256].into_boxed_slice(),
            ram: vec![0u16; 65536].into_boxed_slice(),
            pc: 0,
            sp: 0,
            ia: 0,
            ex: 0
        }
    }

    pub fn load_program(&mut self, program: &[u16]) -> &mut VirtualMachine {
        self.reset();
        let mut i = 0;
        for word in program {
            self.ram[i] = *word;
            i = i + 1;
        }

        self
    }

    pub fn reset(&mut self) {
        self.pc = 0;
        self.sp = 0;
        self.ia = 0;
        self.ex = 0;
        for b in (*self.registers).iter_mut() {
            *b = 0;
        }
        for b in (*self.ram).iter_mut() {
            *b = 0;
        }
        for b in (*self.stack).iter_mut() {
            *b = 0;
        }
    }
}

