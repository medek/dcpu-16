use opcodes::{Opcode, Operand, Disassemble};
use virtual_machine::Register;
use result::{DcpuResult, DcpuError, DcpuErrorKind};

const A_MASK:u16 = 0xfc00;
const B_MASK:u16 = 0x03e0;

fn int_to_register(i: u16) -> Register {
    match i {
        0 => Register::A,
        1 => Register::B,
        2 => Register::C,
        3 => Register::X,
        4 => Register::Y,
        5 => Register::Z,
        6 => Register::I,
        7 => Register::J,
        _ => unreachable!()
    }
}

fn from_short_literal(n: u16) -> u16 {
    ((n as i16) - 0x21) as u16
}

fn get_operand(is_a: bool, inst: u16, next: Option<&u16>) -> DcpuResult<(Operand, bool)> {
    let op = match is_a {
        true => (inst & A_MASK) >> 10,
        false => (inst & B_MASK) >> 5,
    };

    match op {
        0x00...0x07 => Ok((Operand::Register(int_to_register(op)),false)),
        0x08...0x0f => Ok((Operand::RegisterDeRef(int_to_register(op - 0x8)), false)),
        0x10...0x17 => {
            if next == None {
                return Err(DcpuError{reason: DcpuErrorKind::MissingNextWord});
            }
            Ok((Operand::RegisterPlusDeRef(int_to_register(op - 0x10), *next.unwrap()), true))
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

impl Disassemble for Vec<u16> {
    fn disassm(&self) -> DcpuResult<Vec<Opcode>> {
        let mut ret = Vec::<Opcode>::new();
        let mut itr = (self).iter().peekable();
        while let Some(curr) = itr.next() {
            if *curr == 0 { break; }
            let peek = itr.peek().cloned();
            match *curr & 0x1F {
                0x00 => {
                    let (op, eat) = try!(handle_special_op(*curr, peek));
                    if eat { itr.next(); }
                    ret.push(op);
                },
                0x01 => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::SET(b, a));
                },
                0x02 => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::ADD(b, a));
                },
                0x03 => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::SUB(b, a));
                },
                0x04 => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::MUL(b, a));
                },
                0x05 => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::MLI(b, a));
                },
                0x06 => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::DIV(b, a));
                },
                0x07 => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::DVI(b, a));
                },
                0x08 => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::MOD(b, a));
                },
                0x09 => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::MDI(b, a));
                },
                0x0a => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::AND(b, a));
                },
                0x0b => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::BOR(b, a));
                },
                0x0c => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::XOR(b, a));
                },
                0x0d => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::SHR(b, a));
                },
                0x0e => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::ASR(b, a));
                },
                0x0f => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::SHL(b, a));
                },
                0x10 => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::IFB(b, a));
                },
                0x11 => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::IFC(b, a));
                },
                0x12 => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::IFE(b, a));
                },
                0x13 => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::IFN(b, a));
                },
                0x14 => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::IFG(b, a));
                },
                0x15 => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::IFA(b, a));
                },
                0x16 => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::IFL(b, a));
                },
                0x17 => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::IFU(b, a));
                },
                0x18...0x19 => {
                    return Err(DcpuError{reason: DcpuErrorKind::ReservedOpcode(*curr)});
                },
                0x1a => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::ADX(b, a));
                },
                0x1b => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::SBX(b, a));
                },
                0x1c...0x1d => {
                    return Err(DcpuError{reason: DcpuErrorKind::ReservedOpcode(*curr)});
                },
                0x1e => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::STI(b, a));
                },
                0x1f => {
                    let (a, eat) = try!(get_operand(true, *curr, peek));
                    if eat { itr.next(); }
                    let (b, eat) = try!(get_operand(false, *curr, peek));
                    if eat { itr.next(); }
                    ret.push(Opcode::STD(b, a));
                }
                _ => unreachable!()
            };
        }
        Ok(ret)
    }
}
