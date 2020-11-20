use opcodes::{Opcode, Operand};
use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum AssemblyError {
    #[error("Invalid operand A. Cannot put PUSH there.")]
    PushInAOp,
    #[error("Invalid operand B. Cannot put POP there.")]
    PopInBOp,
    #[error("Label not resolved to literal yet: {}", .0)]
    CannotAssembleLabels(String)
}

pub trait Assemble {
    fn assemble(&self) -> Result<Vec<u16>, AssemblyError>;
}

fn is_short_literal(n: u16) -> bool {
    if n as i16 >= -1 && n as i16 <= 30 { return true; }
    false
}

// 0x20-0x3f = -1 to 30 (0xFFFF to 0x1e) for a part
fn to_short_literal(n: u16) -> u16 {
        (0x21u32 + (n as u32)) as u16
}

fn build_operand(is_a: bool, op: &Operand) -> Result<(u16, Option<u16>), AssemblyError> {
    let shift = match is_a {
        true => 10,
        false => 5
    };
    match *op {
        Operand::Register(ref reg) =>
            Ok(((*reg as u16) << shift, None)),
        Operand::RegisterDeref(ref reg) =>
            Ok(((*reg as u16 + 0x8) << shift, None)),
        Operand::RegisterPlusDeref(ref reg, ref lit) =>
            Ok(((*reg as u16 + 0x10) << shift, Some(*lit))),
        Operand::Pop => {
            if is_a {
                Ok((0x18 << shift, None))
            }
            else {
                Err(AssemblyError::PopInBOp)
            }
        },
        Operand::Push => {
            if !is_a {
                Ok((0x18 << shift, None))
            }
            else {
                Err(AssemblyError::PushInAOp)
            }
        },
        Operand::Peek =>
            Ok((0x19 << shift, None)),
        Operand::Pick(ref n) =>
            Ok((0x1a << shift, Some(*n))),
        Operand::Sp =>
            Ok((0x1b << shift, None)),
        Operand::Pc =>
            Ok((0x1c << shift, None)),
        Operand::Ex =>
            Ok((0x1d << shift, None)),
        Operand::LiteralDeref(ref lit) =>
            Ok((0x1e << shift, Some(*lit))),
        Operand::Literal(ref lit) => {
            if is_a && is_short_literal(*lit) {
                Ok((to_short_literal(*lit) << shift, None))
            }
            else {
                Ok((0x1f << shift, Some(*lit)))
            }
        },
        Operand::Label(ref s) => {
            Err(AssemblyError::CannotAssembleLabels(s.clone()))
        },
        Operand::LabelDeref(ref s) => {
            Err(AssemblyError::CannotAssembleLabels(s.clone()))
        },
        Operand::LabelPlusDeref(ref s, _) => {
            Err(AssemblyError::CannotAssembleLabels(s.clone()))
        },
        Operand::LabelPlusLabelDeref(ref s1, ref _s2) => {
            Err(AssemblyError::CannotAssembleLabels(s1.clone()))
        },
        _ => unimplemented!()
    }
}

fn build_op(opcode: u16, b: &Operand, a: &Operand) -> Result<Vec<u16>, AssemblyError> {
    let mut op = opcode & 0x1f;
    let mut ret = Vec::<u16>::new();

    op = op | match build_operand(true, a) {
        Ok((mask, Some(lit))) => {
            ret.push(lit);
            mask
        },
        Ok((mask, None)) => mask,
        Err(err) => return Err(err)
    };

    op = op | match build_operand(false, b) {
        Ok((mask, Some(lit))) => {
            ret.push(lit);
            mask
        },
        Ok((mask, None)) => mask,
        Err(err) => return Err(err)
    };
    ret.insert(0, op);
    Ok(ret)
}

fn build_special_op(opcode: u16, a: &Operand) -> Result<Vec<u16>, AssemblyError> {
    let mut op = (opcode & 0x1f) << 5 ;
    let mut ret = Vec::<u16>::new();

    op = op | match build_operand(true, a) {
        Ok((mask, Some(lit))) => {
            ret.push(lit);
            mask
        },
        Ok((mask, None)) => mask,
        Err(err) => return Err(err)
    };
    ret.insert(0, op);
    Ok(ret)
}

impl Assemble for Opcode {
    fn assemble(&self) -> Result<Vec<u16>, AssemblyError> {
        match *self {
            Opcode::SET(ref b, ref a) => build_op(0x01, b, a),
            Opcode::ADD(ref b, ref a) => build_op(0x02, b, a),
            Opcode::SUB(ref b, ref a) => build_op(0x03, b, a),
            Opcode::MUL(ref b, ref a) => build_op(0x04, b, a),
            Opcode::MLI(ref b, ref a) => build_op(0x05, b, a),
            Opcode::DIV(ref b, ref a) => build_op(0x06, b, a),
            Opcode::DVI(ref b, ref a) => build_op(0x07, b, a),
            Opcode::MOD(ref b, ref a) => build_op(0x08, b, a),
            Opcode::MDI(ref b, ref a) => build_op(0x09, b, a),
            Opcode::AND(ref b, ref a) => build_op(0x0a, b, a),
            Opcode::BOR(ref b, ref a) => build_op(0x0b, b, a),
            Opcode::XOR(ref b, ref a) => build_op(0x0c, b, a),
            Opcode::SHR(ref b, ref a) => build_op(0x0d, b, a),
            Opcode::ASR(ref b, ref a) => build_op(0x0e, b, a),
            Opcode::SHL(ref b, ref a) => build_op(0x0f, b, a),
            Opcode::IFB(ref b, ref a) => build_op(0x10, b, a),
            Opcode::IFC(ref b, ref a) => build_op(0x11, b, a),
            Opcode::IFE(ref b, ref a) => build_op(0x12, b, a),
            Opcode::IFN(ref b, ref a) => build_op(0x13, b, a),
            Opcode::IFG(ref b, ref a) => build_op(0x14, b, a),
            Opcode::IFA(ref b, ref a) => build_op(0x15, b, a),
            Opcode::IFL(ref b, ref a) => build_op(0x16, b, a),
            Opcode::IFU(ref b, ref a) => build_op(0x17, b, a),
            Opcode::ADX(ref b, ref a) => build_op(0x1a, b, a),
            Opcode::SBX(ref b, ref a) => build_op(0x1b, b, a),
            Opcode::STI(ref b, ref a) => build_op(0x1e, b, a),
            Opcode::STD(ref b, ref a) => build_op(0x1f, b, a),
            Opcode::JSR(ref a) => build_special_op(0x01, a),
            Opcode::INT(ref a) => build_special_op(0x08, a),
            Opcode::IAG(ref a) => build_special_op(0x09, a),
            Opcode::IAS(ref a) => build_special_op(0x0a, a),
            Opcode::RFI(ref a) => build_special_op(0x0b, a),
            Opcode::IAQ(ref a) => build_special_op(0x0c, a),
            Opcode::HWN(ref a) => build_special_op(0x10, a),
            Opcode::HWQ(ref a) => build_special_op(0x11, a),
            Opcode::HWI(ref a) => build_special_op(0x12, a)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn short_literal() {
        assert_eq!(build_operand(true, &Operand::Literal(0xFFFF)), Ok((0x20<<10, None)))
    }
}