use std::fmt::{Display, Formatter, Error};
use opcodes::{Opcode, Operand};
use result::{DcpuResult, DcpuError, DcpuErrorKind};
use disassemble::disassm_one;
use mem_iterator::MemIterator;
use hardware::{Hardware, Hw, RealtimeClock};
use std::borrow::BorrowMut;

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
    cycles: usize,
    clock_rate: usize,
    hardware: Vec<Box<Hw>>
}

fn normalize_stack_address(n: u16) -> u16 {
    if n < 256 {
        n
    }
    else {
        255 - (n & 0xFF)
    }
}

impl<'r> VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine{
            registers: vec![0u16; 8].into_boxed_slice(),
            stack: vec![0u16; 256].into_boxed_slice(),
            ram: Box::<Vec<u16>>::new(vec![0u16; 65536]),
            pc: 0,
            sp: 255,
            ia: 0,
            ex: 0,
            dead_zone: 0,
            cycles: 0,
            clock_rate: 10000, // default to 10KHz
            hardware: Vec::<Box<Hw>>::new()
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

    fn skip(&'r mut self, off: usize) -> DcpuResult<(usize, usize)> {
        let mut skipped:usize = 0;
        let mut count:usize = 0;
        let mut itr = MemIterator::new(&*self.ram, off + self.pc as usize, 0xFFFF).peekable();

        loop {
            let inst = match itr.next() {
                Some(i) => *i,
                None => return Err(DcpuError{reason: DcpuErrorKind::EmptyIterator})
            };
            let o = inst & 0x1f;
            let (op, c) = try!(disassm_one(inst, &mut itr));
            count += c + 1;
            skipped += 1;
            if o < 0x10 && o > 0x17 { break; }
        }

        Ok((skipped, count))
    }

    fn get_instruction(&'r mut self) -> DcpuResult<(Opcode, usize)> {
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
                cycles += c;
                let (dst, c) = try!(self.resolve_memory_write(b));
                *dst = src;
                cycles += c + 1;
            },
            Opcode::ADD(ref b, ref a) => {
                let mut res:u32;

                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
                    cycles += c + 2;
                    res = *dst as u32 + src as u32;
                    *dst = (res & 0xFFFF) as u16;
                }

                if res > 0xFFFF {
                    self.ex = 1;
                }
            },
            Opcode::SUB(ref b, ref a) => {
                let mut res:i32;

                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
                    cycles += c + 2;
                    res = *dst as i32 - src as i32;
                    *dst = (res & 0xFFFF) as u16;
                }

                if res < 0 {
                    self.ex = 0xFFFF;
                }
            },
            Opcode::MUL(ref b, ref a) => {
                let mut res:u32;

                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
                    cycles += c + 2;
                    res = *dst as u32 * src as u32;
                    *dst = (res & 0xFFFF) as u16;
                }
                self.ex = (res>>16) as u16;
            },
            Opcode::MLI(ref b, ref a) => {
                let mut res:i32;

                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
                    cycles += c + 2;
                    res = (*dst as i16) as i32 * ((src as i16) as i32);
                    *dst = (res & 0xFFFF) as u16;
                }
                self.ex = ((res >> 16) & 0xFFFF) as u16;
            },
            Opcode::DIV(ref b, ref a) => {
                let mut res:u32;

                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
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
                let mut res:i32;

                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
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
                let (src, c) = try!(self.resolve_memory_read(a));
                cycles += c;
                let (dst, c) = try!(self.resolve_memory_write(b));
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
                let (src, c) = try!(self.resolve_memory_read(a));
                cycles += c;
                let (dst, c) = try!(self.resolve_memory_write(b));
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
                let (src, c) = try!(self.resolve_memory_read(a));
                cycles += c;
                let (dst, c) = try!(self.resolve_memory_write(b));
                cycles += c + 1;
                *dst = *dst & src;
            },
            Opcode::BOR(ref b, ref a) => {
                let (src, c) = try!(self.resolve_memory_read(a));
                cycles += c;
                let (dst, c) = try!(self.resolve_memory_write(b));
                cycles += c + 1;
                *dst = *dst | src;
            },
            Opcode::XOR(ref b, ref a) => {
                let (src, c) = try!(self.resolve_memory_read(a));
                cycles += c;
                let (dst, c) = try!(self.resolve_memory_write(b));
                cycles += c + 1;
                *dst = *dst ^ src;
            },
            Opcode::SHR(ref b, ref a) => {
                let mut res:u32;
                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
                    cycles += c + 1;
                    *dst = *dst >> src;
                    res = ((*dst as u32) << 16)>> (src as u32) & 0xFFFF;
                }
                self.ex = res as u16;
            },
            Opcode::ASR(ref b, ref a) => {
                let mut res:i32;
                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
                    cycles += c + 1;
                    *dst = ((*dst as i16) >> src) as u16;
                    res = (((*dst as i32) << src as i32)>> 16) & 0xFFFF;
                }
                self.ex = res as u16;
            },
            Opcode::SHL(ref b, ref a) => {
                let mut res:u32;
                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
                    cycles += c + 1;
                    *dst = *dst << src;
                    res = (((*dst as u32) << (src as u32)) >> 16) & 0xFFFF;
                }
                self.ex = res as u16;

            },
            Opcode::IFB(ref b, ref a) => {
                let mut pass:bool;
                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
                    cycles += c + 2;
                    pass = *dst & src != 0;
                }

                if !pass {
                    let (skip, c) = try!(self.skip(count + 1));

                    cycles += skip + 1; // +1 cause failed
                    self.pc =  (self.pc + (c as u16)) & 0xFFFF;
                }
            },
            Opcode::IFC(ref b, ref a) => {
                let mut pass:bool;
                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
                    cycles += c + 2;
                    pass = *dst & src == 0;
                }

                if !pass {
                    let (skip, c) = try!(self.skip(count + 1));

                    cycles += skip + 1; // +1 cause failed
                    self.pc =  (self.pc + (c as u16)) & 0xFFFF;
                }
            },
            Opcode::IFE(ref b, ref a) => {
                let mut pass:bool;
                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
                    cycles += c + 2;
                    pass = *dst == src;
                }

                if !pass {
                    let (skip, c) = try!(self.skip(count + 1));

                    cycles += skip + 1; // +1 cause failed
                    self.pc =  (self.pc + (c as u16)) & 0xFFFF;
                }
            },
            Opcode::IFN(ref b, ref a) => {
                let mut pass:bool;
                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
                    cycles += c + 2;
                    pass = *dst != src;
                }

                if !pass {
                    let (skip, c) = try!(self.skip(count + 1));

                    cycles += skip + 1; // +1 cause failed
                    self.pc =  (self.pc + (c as u16)) & 0xFFFF;
                }
            },
            Opcode::IFG(ref b, ref a) => {
                let mut pass:bool;
                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
                    cycles += c + 2;
                    pass = *dst > src;
                }

                if !pass {
                    let (skip, c) = try!(self.skip(count + 1));

                    cycles += skip + 1; // +1 cause failed
                    self.pc =  (self.pc + (c as u16)) & 0xFFFF;
                }
            },
            Opcode::IFA(ref b, ref a) => {
                let mut pass:bool;
                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
                    cycles += c + 2;
                    pass = (*dst as i16) > (src as i16);
                }

                if !pass {
                    let (skip, c) = try!(self.skip(count + 1));

                    cycles += skip + 1; // +1 cause failed
                    self.pc =  (self.pc + (c as u16)) & 0xFFFF;
                }
            },
            Opcode::IFL(ref b, ref a) => {
                let mut pass:bool;
                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
                    cycles += c + 2;
                    pass = *dst < src;
                }

                if !pass {
                    let (skip, c) = try!(self.skip(count + 1));

                    cycles += skip + 1; // +1 cause failed
                    self.pc =  (self.pc + (c as u16)) & 0xFFFF;
                }
            },
            Opcode::IFU(ref b, ref a) => {
                let mut pass:bool;
                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
                    cycles += c + 2;
                    pass = (*dst as i16) < (src as i16);
                }

                if !pass {
                    let (skip, c) = try!(self.skip(count + 1));

                    cycles += skip + 1; // +1 cause failed
                    self.pc =  (self.pc + (c as u16)) & 0xFFFF;
                }
            },
            Opcode::ADX(ref b, ref a) => {
                let mut res:u32 = self.ex as u32;

                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
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
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
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
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
                    cycles += c + 2;
                    *dst = src;
                }
                self.registers[Register::I as usize] += 1;
                self.registers[Register::J as usize] += 1;
            },
            Opcode::STD(ref b, ref a) => {
                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c;
                    let (dst, c) = try!(self.resolve_memory_write(b));
                    cycles += c + 2;
                    *dst = src;
                }
                self.registers[Register::I as usize] -= 1;
                self.registers[Register::J as usize] -= 1;
            },
            Opcode::JSR(ref a) => {
                let mut x:u32 = (self.pc as u32 + count as u32 + 1) & 0xFFFF;
                {
                    let (src, c) = try!(self.resolve_memory_read(a));
                    cycles += c + 3;
                    let (dst, _) = try!(self.resolve_memory_write(&Operand::Push));
                    *dst = x as u16;
                    x = src as u32;
                }
                self.pc = x as u16;
            },
            Opcode::INT(ref a) => {
                unimplemented!()
            },
            Opcode::IAG(ref a) => {
                unimplemented!()
            },
            Opcode::IAS(ref a) => {
                unimplemented!()
            },
            Opcode::RFI(ref a) => {
                unimplemented!()
            },
            Opcode::IAQ(ref a) => {
                unimplemented!()
            },
            Opcode::HWN(ref a) => {
                unimplemented!()
            },
            Opcode::HWQ(ref a) => {
                unimplemented!()
            },
            Opcode::HWI(ref a) => {
                unimplemented!()
            },
        }
        self.cycles = self.cycles + cycles;
        self.pc = ((self.pc as u32 + (count as u32) + 1) & 0xFFFF) as u16;
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
            self.ram[i] = *word;
            i = i + 1;
        }
        self
    }

    pub fn attach_hardware(mut self, hardware: Box<Hw>) -> Self {
        self.hardware.push(hardware);
        self
    }

    pub fn get_ram(&'r mut self) -> &'r mut Vec<u16> {
        &mut *self.ram
    }

    pub fn get_registers(&'r mut self) -> &'r mut [u16] {
        &mut *self.registers
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
        self.clock_rate
    }

    pub fn clock_rate(mut self, cr: usize) -> Self {
        self.clock_rate = cr;
        self
    }

    pub fn get_cycles(&'r self) -> usize {
        self.cycles
    }

    pub fn update_hardware(&mut self) {
        for mut hw in self.hardware {
            hw.update(self);
        }
    }

    pub fn interrupt(&'r mut self, msg: u16) {
        unimplemented!()
    }

    pub fn reset(&'r mut self) {
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

