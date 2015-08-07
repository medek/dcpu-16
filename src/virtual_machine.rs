
use opcodes::{Opcode, Operand};
use result::{DcpuResult, DcpuError, DcpuErrorKind};
use disassemble::disassm_one;

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
    registers: Box<[u16]>,
    stack: Box<[u16]>,
    ram: Box<Vec<u16>>,
    pc: u16,
    sp: u16,
    ia: u16,
    ex: u16,
    dead_zone: u16, //where writing to literals goes to die
    cycles: usize
}

fn normalize_stack_address(n: u16) -> u16 {
    if n < 256 {
        n
    }
    else {
        255 - n & 0xFF
    }
}

impl<'r> VirtualMachine {
    pub fn new() -> VirtualMachine {
        VirtualMachine{
            registers: vec![0u16; 8].into_boxed_slice(),
            stack: vec![0u16; 256].into_boxed_slice(),
            ram: Box::<Vec<u16>>::new(vec![0u16; 65536]),
            pc: 0,
            sp: 255,
            ia: 0,
            ex: 0,
            dead_zone: 0,
            cycles: 0
        }
    }

    fn get_instruction(&'r mut self) -> (u16, Option<u16>, Option<u16>) {
        let mut pc = self.pc as usize;
        let inst = (*(self.ram))[pc & 0xFFFF];
        pc = pc + 1;
        let next_a = Some((*(self.ram))[pc & 0xFFFF]);
        pc = pc + 1;
        let next_b = Some((*(self.ram))[pc & 0xFFFF]);
        (inst, next_a, next_b)
    }

    fn resolve_memory_read(&mut self, op: &Operand) -> DcpuResult<(u16, usize)> {
        match *op {
            Operand::Register(reg) => Ok(((*(self.registers))[reg as usize], 0)),
            Operand::RegisterDeRef(reg) => {
                let addr = (*(self.registers))[reg as usize];
                Ok(((*(self.ram))[addr as usize], 0))
            },
            Operand::RegisterPlusDeRef(reg, plus) => {
                let addr = (*(self.registers))[reg as usize] + plus;
                Ok(((*(self.ram))[addr as usize], 1))
            },
            Operand::Peek => {
                Ok(((*(self.stack))[normalize_stack_address(self.sp) as usize], 0))
            },
            Operand::Pick(n) => {
                Ok(((*(self.stack))[normalize_stack_address(self.sp + n) as usize], 1))
            },
            Operand::Pc => {
                Ok((self.pc, 0))
            },
            Operand::Sp => {
                Ok((self.sp, 0))
            },
            Operand::Ex => {
                Ok((self.ex, 0))
            },
            Operand::LiteralDeRef(n) => {
                Ok(((*(self.ram))[n as usize], 1))
            },
            Operand::Literal(n) => {
                Ok((n, 1))
            },
            Operand::Pop => {
                let n = (*(self.stack))[normalize_stack_address(self.sp) as usize];
                self.sp = self.sp + 1;
                Ok((n, 0))
            },
            Operand::Push => {
                Err(DcpuError{reason: DcpuErrorKind::PushInAOp})
            }
        }
    }

    fn resolve_memory_write(&'r mut self, op: &Operand) -> DcpuResult<(&'r mut u16, usize)> {
        match *op {
            Operand::Register(reg) => Ok((&mut (*(self.registers))[reg as usize], 0)),
            Operand::RegisterDeRef(reg) => {
                let addr = (*(self.registers))[reg as usize];
                Ok((&mut(*(self.ram))[addr as usize], 0))
            },
            Operand::RegisterPlusDeRef(reg, plus) => {
                let addr = (*(self.registers))[reg as usize] + plus;
                Ok((&mut (*(self.ram))[addr as usize], 1))
            },
            Operand::Peek => {
                Ok((&mut (*(self.stack))[normalize_stack_address(self.sp) as usize], 0))
            },
            Operand::Pick(n) => {
                Ok((&mut (*(self.stack))[normalize_stack_address(self.sp + n) as usize], 1))
            },
            Operand::Pc => {
                Ok((&mut self.pc, 0))
            },
            Operand::Sp => {
                Ok((&mut self.sp, 0))
            },
            Operand::Ex => {
                Ok((&mut self.ex, 0))
            },
            Operand::LiteralDeRef(n) => {
                Ok((&mut (*(self.ram))[n as usize], 1))
            },
            Operand::Literal(n) => {
                Ok((&mut self.dead_zone, 1))
            },
            Operand::Pop => {
                Err(DcpuError{reason: DcpuErrorKind::PopInBOp})
            },
            Operand::Push => {
                self.sp = self.sp + 1;
                Ok((&mut (*(self.stack))[normalize_stack_address(self.sp) as usize], 0))
            }
        }
    }

    pub fn step(&'r mut self) -> DcpuResult<usize> {
        let (inst, next_a, next_b) = self.get_instruction();
        let (op, count) = try!(disassm_one(inst, next_a.as_ref(), next_b.as_ref()));
        let mut cycles:usize = 0;
        match op {
            Opcode::SET(ref b, ref a) => {
                let (src, c) = try!(self.resolve_memory_read(a));
                cycles = cycles + c;
                let (dst, c) = try!(self.resolve_memory_write(b));
                cycles = cycles + c;
                *dst = src;
                cycles = cycles + 1; //SET uses 1 cycle
            }
            _ => unimplemented!()
        }
        self.cycles = self.cycles + cycles;
        self.pc = self.pc + (count as u16) + 1;
        Ok(cycles)
    }

    pub fn set_pc(&mut self, pc: u16) -> &mut VirtualMachine {
        self.pc = pc;
        self
    }

    pub fn set_sp(&mut self, sp: u16) -> &mut VirtualMachine {
        self.sp = sp;
        self
    }

    pub fn load_program(&mut self, program: &Vec<u16>, org: usize) -> &mut VirtualMachine {
        self.reset();
        let mut i = org;

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

