use std::fmt::{Display, Formatter, Error};
use opcodes::{Opcode, Operand};
use disassemble::{disassm_one, DcpuDisassmError};
use mem_iterator::MemIterator;
use hardware::{Hardware};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DcpuVMError {
    #[error("Invalid operand A. Cannot put PUSH there.")]
    PushInAOp,
    #[error("Invalid operand B. Cannot put POP there.")]
    PopInBOp,
    #[error("0x00 isn't a valid instruction")]
    EmptyInstruction,
    #[error("Address out of bounds. Maybe external hardware tried to read too much RAM")]
    OutOfBoundsMemory,
    #[error("Shit's on fire, yo!")]
    OnFire,
    #[error("Someone passed in an empty iterator!")]
    EmptyIterator,
    #[error("Disassembly error: {}", .0)]
    DisassemblyFailed(#[from]DcpuDisassmError)
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Register {
    A = 0,
    B,
    C,
    X,
    Y,
    Z,
    I,
    J,
}

impl Register {
    pub fn from_str(s: &String) -> Option<Register> {
        if s.len() > 1 || s.len() == 0 { return None }
        let u = s.to_uppercase();
        match u.as_str().as_bytes()[0] {
            b'A' => Some(Register::A),
            b'B' => Some(Register::B),
            b'C' => Some(Register::C),
            b'X' => Some(Register::X),
            b'Y' => Some(Register::Y),
            b'Z' => Some(Register::Z),
            b'I' => Some(Register::I),
            b'J' => Some(Register::J),
            _ => None
        }
    }
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
            Register::J => fmt.write_str("J"),
        }
    }
}

#[derive(Debug)]
pub struct VMExposed {
    registers: Box<[u16]>,
    ram: Box<Vec<u16>>,
    interrupts: Vec<u16>,
    cycles: usize,
    clock_rate: usize,
}

#[derive(Debug)]
pub struct VirtualMachine {
    exposed: VMExposed,
    pc: u16,
    sp: u16,
    ia: u16,
    ex: u16,
    dead_zone: u16, //where writing to literals goes to die
    hardware: Vec<Box<dyn Hardware>>,
    in_interrupt: bool,
    iaq: bool,
    on_fire: bool
}

fn rollover_inc(i: u16) -> u16 {
    ((i as u32 + 1) & 0xFFFF) as u16
}

fn rollover_dec(i: u16) -> u16 {
    if i == 0 {
        return 0xFFFF
    }
    i - 1
}

impl<'r> VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine{
            exposed: VMExposed {
                registers: vec![0u16; 8].into_boxed_slice(),
                ram: Box::<Vec<u16>>::new(vec![0u16; 65536]),
                interrupts: Vec::<u16>::new(),
                cycles: 0,
                clock_rate: 100000, // default to 100KHz
            },
            pc: 0,
            sp: 0,
            ia: 0,
            ex: 0,
            dead_zone: 0,
            hardware: Vec::<Box<dyn Hardware>>::new(),
            iaq: false,
            in_interrupt: false,
            on_fire: false
        }
    }

    fn push_stack(&mut self, data: u16) {
        self.sp = rollover_dec(self.sp);
        self.exposed.ram[self.sp as usize] = data;
    }

    fn pop_stack(&mut self) -> u16 {
        let ret = self.exposed.ram[self.sp as usize];
        self.sp = rollover_inc(self.sp);
        ret
    }

    fn resolve_memory_read(&mut self, op: &Operand) -> Result<(u16, usize), DcpuVMError> {
        match *op {
            Operand::Register(reg) => Ok(((*(self.exposed.registers))[reg as usize], 0)),
            Operand::RegisterDeref(reg) => {
                let addr = (*(self.exposed.registers))[reg as usize];
                Ok(((*(self.exposed.ram))[addr as usize], 0))
            },
            Operand::RegisterPlusDeref(reg, plus) => {
                let addr = (*(self.exposed.registers))[reg as usize] + plus;
                Ok(((*(self.exposed.ram))[addr as usize], 1))
            },
            Operand::Peek => {
                Ok((self.exposed.ram[self.sp as usize], 0))
            },
            Operand::Pick(n) => {
                Ok((self.exposed.ram[((self.sp + n) & 0xFFFF) as usize], 1))
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
            Operand::LiteralDeref(n) => {
                Ok(((*(self.exposed.ram))[n as usize], 1))
            },
            Operand::Literal(n) => {
                Ok((n, 1))
            },
            Operand::Pop => {
                let n = self.pop_stack();
                Ok((n, 0))
            },
            Operand::Push => {
                Err(DcpuVMError::PushInAOp)
            },
            _ => unreachable!()
        }
    }

    fn resolve_memory_write(&'r mut self, op: &Operand) -> Result<(&'r mut u16, usize), DcpuVMError> {
        match *op {
            Operand::Register(reg) => Ok((&mut (*(self.exposed.registers))[reg as usize], 0)),
            Operand::RegisterDeref(reg) => {
                let addr = (*(self.exposed.registers))[reg as usize];
                Ok((&mut(*(self.exposed.ram))[addr as usize], 0))
            },
            Operand::RegisterPlusDeref(reg, plus) => {
                let addr = (*(self.exposed.registers))[reg as usize] + plus;
                Ok((&mut (*(self.exposed.ram))[addr as usize], 1))
            },
            Operand::Peek => {
                Ok((&mut self.exposed.ram[self.sp as usize], 0))
            },
            Operand::Pick(n) => {
                Ok((&mut self.exposed.ram[((self.sp as u32 + n as u32) & 0xFFFF) as usize], 1))
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
            Operand::LiteralDeref(n) => {
                Ok((&mut (*(self.exposed.ram))[n as usize], 1))
            },
            Operand::Literal(_) => {
                Ok((&mut self.dead_zone, 1))
            },
            Operand::Pop => {
                Err(DcpuVMError::PopInBOp)
            },
            Operand::Push => {
                self.sp = ((self.sp as u32 - 1) & 0xFFFF) as u16;
                let ret = &mut self.exposed.ram[self.sp as usize];
                Ok((ret, 0))
            },
            _ => unreachable!()
        }
    }

    fn skip(&'r mut self, off: usize) -> Result<(usize, usize), DcpuVMError> {
        let mut skipped:usize = 0;
        let mut count:usize = 0;
        let mut itr = MemIterator::new(&*self.exposed.ram, off + self.pc as usize, 0xFFFF).peekable();

        loop {
            let inst = match itr.next() {
                Some(i) => *i,
                None => return Err(DcpuVMError::EmptyIterator)
            };
            let o = inst & 0x1f;
            let (_, c) = disassm_one(inst, &mut itr)?;
            count += c + 1;
            skipped += 1;
            if o < 0x10 && o > 0x17 { break; }
        }

        Ok((skipped, count))
    }

    fn handle_interrupts(&'r mut self) -> Result<usize, DcpuVMError> {
        if self.ia == 0  || self.iaq {
            return Ok(0)
        }

        if self.on_fire {
            return Err(DcpuVMError::OnFire)
        }

        if self.exposed.interrupts.len() == 0 {
            return Ok(0)
        }

        let int = self.exposed.interrupts.remove(0);
        let a = self.exposed.registers[Register::A as usize];
        let pc = self.pc;
        self.push_stack(pc);
        self.push_stack(a);
        self.pc = self.ia;
        self.exposed.registers[Register::A as usize] = int;
        self.in_interrupt = true;
        self.step()
    }

    fn get_instruction(&'r mut self) -> Result<(Opcode, usize), DcpuVMError> {
        let mut itr = MemIterator::new(&*self.exposed.ram, self.pc as usize, 0xFFFF).peekable();
        let inst = match itr.next() {
            Some(i) => *i,
            None => return Err(DcpuVMError::EmptyIterator)
        };
        Ok(disassm_one(inst, &mut itr)?)
    }

    pub fn step(&'r mut self) -> Result<usize, DcpuVMError> {
        let mut cycles:usize = 0;
        let (op, count) = self.get_instruction()?;
        match op {
            Opcode::SET(ref b, ref a) => {
                let (src, c) = self.resolve_memory_read(a)?;
                cycles += c;
                let (dst, c) = self.resolve_memory_write(b)?;
                *dst = src;
                cycles += c + 1;
            },
            Opcode::ADD(ref b, ref a) => {
                let res:u32;

                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 2;
                    res = *dst as u32 + src as u32;
                    *dst = (res & 0xFFFF) as u16;
                }

                if res > 0xFFFF {
                    self.ex = 1;
                }
            },
            Opcode::SUB(ref b, ref a) => {
                let res:i32;

                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 2;
                    res = *dst as i32 - src as i32;
                    *dst = (res & 0xFFFF) as u16;
                }

                if res < 0 {
                    self.ex = 0xFFFF;
                }
            },
            Opcode::MUL(ref b, ref a) => {
                let res:u32;

                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 2;
                    res = *dst as u32 * src as u32;
                    *dst = (res & 0xFFFF) as u16;
                }
                self.ex = (res>>16) as u16;
            },
            Opcode::MLI(ref b, ref a) => {
                let res:i32;

                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 2;
                    res = (*dst as i16) as i32 * ((src as i16) as i32);
                    *dst = (res & 0xFFFF) as u16;
                }
                self.ex = ((res >> 16) & 0xFFFF) as u16;
            },
            Opcode::DIV(ref b, ref a) => {
                let res:u32;

                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 3;
                    if src == 0 {
                        res = 0;
                        *dst = 0;
                    }
                    else {
                        res = (((*dst as u32) << 16)/src as u32)&0xFFFF;
                        *dst = (*dst as u32 / src as u32) as u16;
                    }
                }
                self.ex = res as u16;
            },
            Opcode::DVI(ref b, ref a) => {
                let res:i32;

                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 3;
                    if src == 0 {
                        res = 0;
                        *dst = 0;
                    }
                    else {
                        res = ((((*dst as i16) as i32) << 16)/((src as i16) as i32))&0xFFFF;
                        *dst = ((*dst as i16) as i32 / ((src as i16) as i32)) as u16;
                    }
                }
                self.ex = res as u16;
            },
            Opcode::MOD(ref b, ref a) => {
                let (src, c) = self.resolve_memory_read(a)?;
                cycles += c;
                let (dst, c) = self.resolve_memory_write(b)?;
                cycles += c + 3;
                if src == 0 {
                    *dst = 0;
                }
                else
                {
                    *dst = *dst % src;
                }
            },
            Opcode::MDI(ref b, ref a) => {
                let (src, c) = self.resolve_memory_read(a)?;
                cycles += c;
                let (dst, c) = self.resolve_memory_write(b)?;
                cycles += c + 3;
                if src == 0 {
                    *dst = 0;
                }
                else
                {
                    *dst = (*dst as i16 % src as i16) as u16;
                }
            },
            Opcode::AND(ref b, ref a) => {
                let (src, c) = self.resolve_memory_read(a)?;
                cycles += c;
                let (dst, c) = self.resolve_memory_write(b)?;
                cycles += c + 1;
                *dst = *dst & src;
            },
            Opcode::BOR(ref b, ref a) => {
                let (src, c) = self.resolve_memory_read(a)?;
                cycles += c;
                let (dst, c) = self.resolve_memory_write(b)?;
                cycles += c + 1;
                *dst = *dst | src;
            },
            Opcode::XOR(ref b, ref a) => {
                let (src, c) = self.resolve_memory_read(a)?;
                cycles += c;
                let (dst, c) = self.resolve_memory_write(b)?;
                cycles += c + 1;
                *dst = *dst ^ src;
            },
            Opcode::SHR(ref b, ref a) => {
                let res:u32;
                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 1;
                    *dst = *dst >> src;
                    res = ((*dst as u32) << 16)>> (src as u32) & 0xFFFF;
                }
                self.ex = res as u16;
            },
            Opcode::ASR(ref b, ref a) => {
                let res:i32;
                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 1;
                    *dst = ((*dst as i16) >> src) as u16;
                    res = (((*dst as i32) << src as i32)>> 16) & 0xFFFF;
                }
                self.ex = res as u16;
            },
            Opcode::SHL(ref b, ref a) => {
                let res:u32;
                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 1;
                    *dst = *dst << src;
                    res = (((*dst as u32) << (src as u32)) >> 16) & 0xFFFF;
                }
                self.ex = res as u16;

            },
            Opcode::IFB(ref b, ref a) => {
                let pass:bool;
                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 2;
                    pass = *dst & src != 0;
                }

                if !pass {
                    let (skip, c) = self.skip(count + 1)?;

                    cycles += skip + 1; // +1 cause failed
                    self.pc =  (self.pc + (c as u16)) & 0xFFFF;
                }
            },
            Opcode::IFC(ref b, ref a) => {
                let pass:bool;
                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 2;
                    pass = *dst & src == 0;
                }

                if !pass {
                    let (skip, c) = self.skip(count + 1)?;

                    cycles += skip + 1; // +1 cause failed
                    self.pc =  (self.pc + (c as u16)) & 0xFFFF;
                }
            },
            Opcode::IFE(ref b, ref a) => {
                let pass:bool;
                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 2;
                    pass = *dst == src;
                }

                if !pass {
                    let (skip, c) = self.skip(count + 1)?;

                    cycles += skip + 1; // +1 cause failed
                    self.pc =  (self.pc + (c as u16)) & 0xFFFF;
                }
            },
            Opcode::IFN(ref b, ref a) => {
                let pass:bool;
                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 2;
                    pass = *dst != src;
                }

                if !pass {
                    let (skip, c) = self.skip(count + 1)?;

                    cycles += skip + 1; // +1 cause failed
                    self.pc =  (self.pc + (c as u16)) & 0xFFFF;
                }
            },
            Opcode::IFG(ref b, ref a) => {
                let pass:bool;
                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 2;
                    pass = *dst > src;
                }

                if !pass {
                    let (skip, c) = self.skip(count + 1)?;

                    cycles += skip + 1; // +1 cause failed
                    self.pc =  (self.pc + (c as u16)) & 0xFFFF;
                }
            },
            Opcode::IFA(ref b, ref a) => {
                let pass:bool;
                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 2;
                    pass = (*dst as i16) > (src as i16);
                }

                if !pass {
                    let (skip, c) = self.skip(count + 1)?;

                    cycles += skip + 1; // +1 cause failed
                    self.pc =  (self.pc + (c as u16)) & 0xFFFF;
                }
            },
            Opcode::IFL(ref b, ref a) => {
                let pass:bool;
                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 2;
                    pass = *dst < src;
                }

                if !pass {
                    let (skip, c) = self.skip(count + 1)?;

                    cycles += skip + 1; // +1 cause failed
                    self.pc =  (self.pc + (c as u16)) & 0xFFFF;
                }
            },
            Opcode::IFU(ref b, ref a) => {
                let pass:bool;
                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 2;
                    pass = (*dst as i16) < (src as i16);
                }

                if !pass {
                    let (skip, c) = self.skip(count + 1)?;

                    cycles += skip + 1; // +1 cause failed
                    self.pc =  (self.pc + (c as u16)) & 0xFFFF;
                }
            },
            Opcode::ADX(ref b, ref a) => {
                let mut res:u32 = self.ex as u32;

                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 3;
                    res += *dst as u32 + src as u32;
                    *dst = (res & 0xFFFF) as u16;
                }

                if res > 0xFFFF {
                    self.ex = 1;
                }
            },
            Opcode::SBX(ref b, ref a) => {
                let mut res:i32 = self.ex as i32;

                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 3;
                    res = *dst as i32 - (src as i32 + res);
                    *dst = (res & 0xFFFF) as u16;
                }

                if res < 0 {
                    self.ex = 0xFFFF;
                }
            },
            Opcode::STI(ref b, ref a) => {
                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 2;
                    *dst = src;
                }
                self.exposed.registers[Register::I as usize] += 1;
                self.exposed.registers[Register::J as usize] += 1;
            },
            Opcode::STD(ref b, ref a) => {
                {
                    let (src, c) = self.resolve_memory_read(a)?;
                    cycles += c;
                    let (dst, c) = self.resolve_memory_write(b)?;
                    cycles += c + 2;
                    *dst = src;
                }
                self.exposed.registers[Register::I as usize] -= 1;
                self.exposed.registers[Register::J as usize] -= 1;
            },
            Opcode::JSR(ref a) => {
                let x:u16 = ((self.pc as u32 + count as u32 + 1) & 0xFFFF) as u16;
                let (src, c) = self.resolve_memory_read(a)?;
                cycles += c + 3;
                self.push_stack(x);
                self.pc = src;
            },
            Opcode::INT(ref a) => {
                let (src, c) = self.resolve_memory_read(a)?;
                cycles += c + 4;
                self.interrupt(src);
            },
            Opcode::IAG(ref a) => {
                let ia:u16 = self.ia;
                let (dst, c) = self.resolve_memory_write(a)?;
                cycles += c + 1;
                *dst = ia;
            },
            Opcode::IAS(ref a) => {
                let (src, c) = self.resolve_memory_read(a)?;
                cycles += c + 1;
                self.ia = src;
            },
            Opcode::RFI(_) => {
                self.iaq = false;
                self.exposed.registers[Register::A as usize] = self.pop_stack();
                self.pc = self.pop_stack();
                self.in_interrupt = false;
                cycles += 3;
            },
            Opcode::IAQ(ref a) => {
                let (src, c) = self.resolve_memory_read(a)?;
                cycles += c + 2;
                self.iaq = src != 0;
            },
            Opcode::HWN(ref a) => {
                let hw_count:u16 = self.hardware.len() as u16;
                let (dst, c) = self.resolve_memory_write(a)?;
                cycles += c + 2;
                *dst = hw_count;
            },
            Opcode::HWQ(ref a) => {
                let (src, c) = self.resolve_memory_read(a)?;
                cycles += c + 4;
                if (src as usize) < self.hardware.len() {
                    let hw_info = self.hardware[src as usize].info();
                    self.exposed.registers[Register::A as usize] = (hw_info.model & 0xFFFF) as u16;
                    self.exposed.registers[Register::B as usize] = (hw_info.model >> 16) as u16;
                    self.exposed.registers[Register::C as usize] = hw_info.version;
                    self.exposed.registers[Register::X as usize] = (hw_info.manufacturer & 0xFFFF) as u16;
                    self.exposed.registers[Register::Y as usize] = (hw_info.manufacturer >> 16) as u16;
                }
            },
            Opcode::HWI(ref a) => {
                let (src, c) = self.resolve_memory_read(a)?;
                cycles += c + 4;
                cycles += self.hardware[src as usize].hardware_interrupt(&mut self.exposed);
            },
        }
        self.exposed.cycles += cycles;
        self.pc = ((self.pc as u32 + (count as u32) + 1) & 0xFFFF) as u16;

        if !self.in_interrupt {
            cycles += self.handle_interrupts()?;
        }
        Ok(cycles)
    }

    pub fn set_pc(mut self, pc: u16) -> Self {
        self.pc = pc;
        self
    }

    pub fn set_sp(mut self, sp: u16) -> Self {
        self.sp = sp;
        self
    }

    pub fn load_program(mut self, program: &Vec<u16>, org: usize) -> Self {
        self.reset();
        let mut i = org;

        for word in program {
            self.exposed.ram[i] = *word;
            i = i + 1;
        }
        self
    }

    pub fn attach_hardware(mut self, hardware: Box<dyn Hardware>) -> Self {
        if self.hardware.len() == 0xFFFF {
            return self
        }
        self.hardware.push(hardware);
        self
    }

    pub fn get_ram(&'r mut self) -> &'r mut Vec<u16> {
        &mut *self.exposed.ram
    }

    pub fn get_registers(&'r mut self) -> &'r mut [u16] {
        &mut *self.exposed.registers
    }

    pub fn get_pc(&'r mut self) -> &'r mut u16 {
        &mut self.pc
    }

    pub fn get_ex(&'r mut self) -> &'r mut u16 {
        &mut self.ex
    }

    pub fn get_sp(&'r mut self) -> &'r mut u16 {
        &mut self.sp
    }

    pub fn get_clock_rate(&'r self) -> usize {
        self.exposed.clock_rate
    }

    pub fn clock_rate(mut self, cr: usize) -> Self {
        self.exposed.clock_rate = cr;
        self
    }

    pub fn get_cycles(&'r self) -> usize {
        self.exposed.cycles
    }

    pub fn update_hardware(&mut self) {
        for mut hw in &mut self.hardware {
            hw.update(&mut self.exposed);
        }
    }

    pub fn interrupt(&'r mut self, msg: u16) {
        if self.ia != 0 && self.on_fire == false {
            self.exposed.interrupt(msg);
        }
    }

    pub fn reset(&'r mut self) {
        self.pc = 0;
        self.sp = 0;
        self.ia = 0;
        self.ex = 0;
        for b in (*self.exposed.registers).iter_mut() {
            *b = 0;
        }
        for b in (*self.exposed.ram).iter_mut() {
            *b = 0;
        }
        self.exposed.cycles = 0;
        self.exposed.interrupts.clear();
        self.in_interrupt = false;
        self.iaq = false;
    }
}

impl VMExposed {
    pub fn interrupt(&mut self, msg: u16) {
        self.interrupts.push(msg);
    }

    pub fn get_cycles(&self) -> usize {
        self.cycles
    }

    pub fn get_clock_rate(&self) -> usize {
        self.clock_rate
    }

    // the cycles I'm using are probably very *very* wrong!

    pub fn read_register(&self, reg: Register) -> (u16, usize) {
        (self.registers[reg as usize], 2) //SET [NEXT], REG
    }

    pub fn write_register(&mut self, reg: Register, data: u16) -> usize {
        self.registers[reg as usize] = data;
        2 //SET REG, LITERAL
    }

    pub fn read_ram<'r> (&'r mut self, pos: usize, size: usize) -> Result<(&'r [u16], usize), DcpuVMError> {
        if pos + size > 0xFFFF {
            return Err(DcpuVMError::OutOfBoundsMemory);
        }
        Ok((&self.ram[(pos) .. (pos + size)], size * 3))
    }

    pub fn write_ram(&mut self, mut pos: usize, data: &[u16], size: usize) -> usize {
        let mut i = 0;
        pos = pos & 0xFFFF;
        loop {
            self.ram[pos] = data[i];
            pos = (pos + 1) & 0xFFFF;
            i += 1;
            if i == size { break; }
        }
        i * 3 //SET [NEXT], LITERAL
    }
}
