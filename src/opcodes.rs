use std::fmt::{Display, Formatter, Error};
use virtual_machine::Register;

#[derive(Debug,PartialEq)]
pub enum Operand {
    Register(Register),
    RegisterDeref(Register),
    RegisterPlusDeref(Register, u16),
    RegisterPlusLabelDeref(Register, String),
    Push,
    Pop,
    Peek,
    Pick(u16),
    Sp,
    Pc,
    Ex,
    Literal(u16),
    LiteralDeref(u16),
    Label(String),
    LabelDeref(String),
    LabelPlusDeref(String, u16),
    LabelPlusLabelDeref(String, String),
}

#[derive(Debug,PartialEq)]
pub enum Opcode {
    SET(Operand, Operand), // SET b, a -> b = a
    ADD(Operand, Operand), // ADD b, a -> b = b+a
    SUB(Operand, Operand), // SUB b, a -> b = b-a
    MUL(Operand, Operand), // MUL b, a -> b = b*a
    MLI(Operand, Operand), // same as MUL but b is signed
    DIV(Operand, Operand), // DIV b, a -> b = b/a
    DVI(Operand, Operand), // same as DIV but signed (round down)
    MOD(Operand, Operand),
    MDI(Operand, Operand),
    AND(Operand, Operand),
    BOR(Operand, Operand),
    XOR(Operand, Operand),
    SHR(Operand, Operand),
    ASR(Operand, Operand),
    SHL(Operand, Operand),
    IFB(Operand, Operand),
    IFC(Operand, Operand),
    IFE(Operand, Operand),
    IFN(Operand, Operand),
    IFG(Operand, Operand),
    IFA(Operand, Operand),
    IFL(Operand, Operand),
    IFU(Operand, Operand),
    ADX(Operand, Operand),
    SBX(Operand, Operand),
    STI(Operand, Operand),
    STD(Operand, Operand),
    JSR(Operand),
    INT(Operand),
    IAG(Operand),
    IAS(Operand),
    RFI(Operand),
    IAQ(Operand),
    HWN(Operand),
    HWQ(Operand),
    HWI(Operand)
}

impl Display for Operand {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            Operand::Register(reg) => {
                reg.fmt(fmt)
            },
            Operand::RegisterDeref(reg) => {
                fmt.write_fmt(format_args!("[{}]", reg))
            },
            Operand::RegisterPlusDeref(reg, n) => {
                fmt.write_fmt(format_args!("[{} + {:#x}]", reg, n))
            },
            Operand::RegisterPlusLabelDeref(reg, ref s) => {
                fmt.write_fmt(format_args!("[{} + {}]", reg, s))
            },
            Operand::Push => {
                fmt.write_str("PUSH")
            },
            Operand::Pop => {
                fmt.write_str("POP")
            },
            Operand::Peek => {
                fmt.write_str("[SP]")
            },
            Operand::Pick(n) => {
                fmt.write_fmt(format_args!("[SP + {:#x}]", n))
            },
            Operand::Pc => {
                fmt.write_str("PC")
            },
            Operand::Sp => {
                fmt.write_str("SP")
            },
            Operand::Ex => {
                fmt.write_str("EX")
            },
            Operand::LiteralDeref(n) => {
                fmt.write_fmt(format_args!("[{:#x}]", n))
            },
            Operand::Literal(n) => {
                fmt.write_fmt(format_args!("{:#x}", n))
            },
            Operand::Label(ref s) => {
                fmt.write_str(s)
            },
            Operand::LabelDeref(ref s) => {
                fmt.write_str(s)
            },
            Operand::LabelPlusDeref(ref s, l) => {
                fmt.write_fmt(format_args!("[{}+{}]", s, l))
            },
            Operand::LabelPlusLabelDeref(ref s, ref l) => {
                fmt.write_fmt(format_args!("[{}+{}]", s, l))
            }
        }
    }
}

impl Display for Opcode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            Opcode::SET(ref b, ref a) => {
                fmt.write_fmt(format_args!("SET {}, {}", b, a))
            },
            Opcode::ADD(ref b, ref a) => {
                fmt.write_fmt(format_args!("ADD {}, {}", b, a))
            },
            Opcode::SUB(ref b, ref a) => {
                fmt.write_fmt(format_args!("SUB {}, {}", b, a))
            },
            Opcode::MUL(ref b, ref a) => {
                fmt.write_fmt(format_args!("MUL {}, {}", b, a))
            },
            Opcode::MLI(ref b, ref a) => {
                fmt.write_fmt(format_args!("MLI {}, {}", b, a))
            },
            Opcode::DIV(ref b, ref a) => {
                fmt.write_fmt(format_args!("DIV {}, {}", b, a))
            },
            Opcode::DVI(ref b, ref a) => {
                fmt.write_fmt(format_args!("DVI {}, {}", b, a))
            },
            Opcode::MOD(ref b, ref a) => {
                fmt.write_fmt(format_args!("MOD {}, {}", b, a))
            },
            Opcode::MDI(ref b, ref a) => {
                fmt.write_fmt(format_args!("MDI {}, {}", b, a))
            },
            Opcode::AND(ref b, ref a) => {
                fmt.write_fmt(format_args!("AND {}, {}", b, a))
            },
            Opcode::BOR(ref b, ref a) => {
                fmt.write_fmt(format_args!("BOR {}, {}", b, a))
            },
            Opcode::XOR(ref b, ref a) => {
                fmt.write_fmt(format_args!("XOR {}, {}", b, a))
            },
            Opcode::SHR(ref b, ref a) => {
                fmt.write_fmt(format_args!("SHR {}, {}", b, a))
            },
            Opcode::ASR(ref b, ref a) => {
                fmt.write_fmt(format_args!("ASR {}, {}", b, a))
            },
            Opcode::SHL(ref b, ref a) => {
                fmt.write_fmt(format_args!("SHL {}, {}", b, a))
            },
            Opcode::IFB(ref b, ref a) => {
                fmt.write_fmt(format_args!("IFB {}, {}", b, a))
            },
            Opcode::IFC(ref b, ref a) => {
                fmt.write_fmt(format_args!("IFC {}, {}", b, a))
            },
            Opcode::IFE(ref b, ref a) => {
                fmt.write_fmt(format_args!("IFE {}, {}", b, a))
            },
            Opcode::IFN(ref b, ref a) => {
                fmt.write_fmt(format_args!("IFN {}, {}", b, a))
            },
            Opcode::IFG(ref b, ref a) => {
                fmt.write_fmt(format_args!("IFG {}, {}", b, a))
            },
            Opcode::IFA(ref b, ref a) => {
                fmt.write_fmt(format_args!("IFA {}, {}", b, a))
            },
            Opcode::IFL(ref b, ref a) => {
                fmt.write_fmt(format_args!("IFL {}, {}", b, a))
            },
            Opcode::IFU(ref b, ref a) => {
                fmt.write_fmt(format_args!("IFU {}, {}", b, a))
            },
            Opcode::ADX(ref b, ref a) => {
                fmt.write_fmt(format_args!("ADX {}, {}", b, a))
            },
            Opcode::SBX(ref b, ref a) => {
                fmt.write_fmt(format_args!("SBX {}, {}", b, a))
            },
            Opcode::STI(ref b, ref a) => {
                fmt.write_fmt(format_args!("STI {}, {}", b, a))
            },
            Opcode::STD(ref b, ref a) => {
                fmt.write_fmt(format_args!("STD {}, {}", b, a))
            },
            Opcode::JSR(ref a) => {
                fmt.write_fmt(format_args!("JSR {}", a))
            },
            Opcode::INT(ref a) => {
                fmt.write_fmt(format_args!("INT {}", a))
            },
            Opcode::IAG(ref a) => {
                fmt.write_fmt(format_args!("IAG {}", a))
            },
            Opcode::IAS(ref a) => {
                fmt.write_fmt(format_args!("IAS {}", a))
            },
            Opcode::RFI(ref a) => {
                fmt.write_fmt(format_args!("RFI {}", a))
            },
            Opcode::IAQ(ref a) => {
                fmt.write_fmt(format_args!("IAQ {}", a))
            },
            Opcode::HWN(ref a) => {
                fmt.write_fmt(format_args!("HWN {}", a))
            },
            Opcode::HWQ(ref a) => {
                fmt.write_fmt(format_args!("HWQ {}", a))
            },
            Opcode::HWI(ref a) => {
                fmt.write_fmt(format_args!("HWI {}", a))
            }
        }
    }
}
