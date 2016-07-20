use opcodes::{Opcode, Operand, Disassemble};
use result::{DcpuResult, DcpuError, DcpuErrorKind};
use std::mem::transmute;
use std::iter::Peekable;
const A_MASK:u16 = 0xfc00;
const B_MASK:u16 = 0x03e0;

fn from_short_literal(n: u16) -> u16 {
    ((n as i16) - 0x21) as u16
}

fn get_operand(is_a: bool, inst: u16, next: Option<&u16>) -> DcpuResult<(Operand, bool)> {
    let op = match is_a {
        true => (inst & A_MASK) >> 10,
        false => (inst & B_MASK) >> 5,
    };

    match op {
        0x00...0x07 => unsafe {Ok((Operand::Register(transmute(op as u32)),false)) },
        0x08...0x0f => unsafe {Ok((Operand::RegisterDeRef(transmute(op as u32 - 0x8)), false)) },
        0x10...0x17 => {
            if next == None {
                return Err(DcpuError{reason: DcpuErrorKind::MissingNextWord});
            }
            unsafe {
                Ok((Operand::RegisterPlusDeRef(transmute(op as u32 - 0x10), *next.unwrap()), true))
            }
        },
        0x18 => {
            if is_a { Ok((Operand::Pop, false)) }
            else { Ok((Operand::Push, false)) }
        },
        0x19 => Ok((Operand::Peek, false)),
        0x1a => {
            if next == None {
                return Err(DcpuError{reason: DcpuErrorKind::MissingNextWord});
            }
            Ok((Operand::Pick(*next.unwrap()), true))
        },
        0x1b => Ok((Operand::Sp, false)),
        0x1c => Ok((Operand::Pc, false)),
        0x1d => Ok((Operand::Ex, false)),
        0x1e => {
            if next == None {
                return Err(DcpuError{reason: DcpuErrorKind::MissingNextWord});
            }
            Ok((Operand::LiteralDeRef(*next.unwrap()), true))
        },
        0x1f => {
            if next == None {
                return Err(DcpuError{reason: DcpuErrorKind::MissingNextWord});
            }
            Ok((Operand::Literal(*next.unwrap()), true))
        },
        0x20...0x3f => Ok((Operand::Literal(from_short_literal(op)),false)),
        _ => unreachable!()
    }
}

fn handle_special_op(inst: u16, next: Option<&u16>) -> DcpuResult<(Opcode, bool)> {
    match (inst & B_MASK) >> 5 {
        0x00 => {
            Err(DcpuError{reason: DcpuErrorKind::ReservedOpcode(inst)})
        },
        0x01 => {
            let (a, eat) = try!(get_operand(true, inst, next));
            Ok((Opcode::JSR(a), eat))
        },
        0x02...0x07 => {
            Err(DcpuError{reason: DcpuErrorKind::ReservedOpcode(inst)})
        },
        0x08 => {
            let (a, eat) = try!(get_operand(true, inst, next));
            Ok((Opcode::JSR(a), eat))
        },
        0x09 => {
            let (a, eat) = try!(get_operand(true, inst, next));
            Ok((Opcode::JSR(a), eat))
        },
        0x0a => {
            let (a, eat) = try!(get_operand(true, inst, next));
            Ok((Opcode::JSR(a), eat))
        },
        0x0b => {
            let (a, eat) = try!(get_operand(true, inst, next));
            Ok((Opcode::JSR(a), eat))
        },
        0x0c => {
            let (a, eat) = try!(get_operand(true, inst, next));
            Ok((Opcode::JSR(a), eat))
        },
        0x0d...0x0f => {
            Err(DcpuError{reason: DcpuErrorKind::ReservedOpcode(inst)})
        },
        0x10 => {
            let (a, eat) = try!(get_operand(true, inst, next));
            Ok((Opcode::JSR(a), eat))
        },
        0x11 => {
            let (a, eat) = try!(get_operand(true, inst, next));
            Ok((Opcode::JSR(a), eat))
        },
        0x12 => {
            let (a, eat) = try!(get_operand(true, inst, next));
            Ok((Opcode::JSR(a), eat))
        },
        0x13...0x1f => {
            Err(DcpuError{reason: DcpuErrorKind::ReservedOpcode(inst)})
        },
        _ => unreachable!()
    }
}

pub fn disassm_one<'a, I>(inst: u16, mut itr: &mut Peekable<I>) -> DcpuResult<(Opcode, usize)> where I: Iterator<Item=&'a u16> {
    let mut count:usize = 0;
    match inst & 0x1F {
        0x00 => {
            let (op, eat) = try!(handle_special_op(inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((op, count))
        },
        0x01 => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::SET(b, a), count))
        },
        0x02 => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::ADD(b, a), count))
        },
        0x03 => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::SUB(b, a), count))
        },
        0x04 => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::MUL(b, a), count))
        },
        0x05 => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::MLI(b, a), count))
        },
        0x06 => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::DIV(b, a), count))
        },
        0x07 => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::DVI(b, a), count))
        },
        0x08 => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::MOD(b, a), count))
        },
        0x09 => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::MDI(b, a), count))
        },
        0x0a => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::AND(b, a), count))
        },
        0x0b => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::BOR(b, a), count))
        },
        0x0c => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::XOR(b, a), count))
        },
        0x0d => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::SHR(b, a), count))
        },
        0x0e => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::ASR(b, a), count))
        },
        0x0f => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::SHL(b, a), count))
        },
        0x10 => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::IFB(b, a), count))
        },
        0x11 => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::IFC(b, a), count))
        },
        0x12 => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::IFE(b, a), count))
        },
        0x13 => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::IFN(b, a), count))
        },
        0x14 => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::IFG(b, a), count))
        },
        0x15 => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::IFA(b, a), count))
        },
        0x16 => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::IFL(b, a), count))
        },
        0x17 => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::IFU(b, a), count))
        },
        0x18...0x19 => {
            return Err(DcpuError{reason: DcpuErrorKind::ReservedOpcode(inst)});
        },
        0x1a => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::ADX(b, a), count))
        },
        0x1b => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::SBX(b, a), count))
        },
        0x1c...0x1d => {
            return Err(DcpuError{reason: DcpuErrorKind::ReservedOpcode(inst)});
        },
        0x1e => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::STI(b, a), count))
        },
        0x1f => {
            let (a, eat) = try!(get_operand(true, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            let (b, eat) = try!(get_operand(false, inst, itr.peek().cloned()));
            if eat {
                itr.next();
                count += 1;
            }
            Ok((Opcode::STD(b, a), count))
        }
        _ => unreachable!()
    }
}

impl Disassemble for Vec<u16> {
    //XXX: this is garbage!
    fn disassm(&self) -> DcpuResult<Vec<Opcode>> {
        let mut ret:Vec<Opcode> = Vec::<Opcode>::new();
        let mut itr = self.iter().peekable();
        while let Some(curr) = itr.next() {
            if *curr == 0 && ret.len() > 0 {
                return Ok(ret);
            }

            match disassm_one(*curr, &mut itr) {
                Ok((op, _)) => {
                    ret.push(op);
                },
                Err(e) => return Err(e)
            };
        }

        if ret.len() > 0 {
            return Ok(ret)
        }
        else {
            return Err(DcpuError{reason: DcpuErrorKind::EmptyIterator})
        }
    }
}
