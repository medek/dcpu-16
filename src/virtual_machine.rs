use std::fmt::{Display, Formatter, Error};
use opcodes::{Opcode, Operand};
use result::{DcpuResult, DcpuError, DcpuErrorKind};
use disassemble::disassm_one;
use mem_iterator::MemIterator;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
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

impl Display for Register {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            Register::A => fmt.write_str("A"),
            Register::B => fmt.write_str("B"),
            Register::C => fmt.write_str("C"),
            Register::X => fmt.write_str("X"),
            Register::Y => fmt.write_str("Y"),
            Register::Z => fmt.write_str("Z"),
            Register::I => fmt.write_str("I"),
            Register::J => fmt.write_str("J")
        }
    }
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
            Operand::Literal(_) => {
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

    pub fn get_instruction(&'r mut self) -> DcpuResult<(Opcode, usize)> {
        let mut itr = MemIterator::new(&*self.ram, self.pc as usize, 0xFFFF).peekable();
        let inst = match itr.next() {
            Some(i) => *i,
            None => return Err(DcpuError{reason: DcpuErrorKind::EmptyIterator})
        };
        disassm_one(inst, &mut itr)
    }

    pub fn step(&'r mut self) -> DcpuResult<usize> {
        let mut cycles:usize = 0;
        let (op, count) = try!(self.get_instruction());
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

