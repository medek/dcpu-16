use std::fmt::{Display, Formatter, Error};
use virtual_machine::Register;

#[derive(Debug,PartialEq,Clone)]
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

#[derive(Debug,PartialEq,Clone)]
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

impl Opcode {
    pub fn set_operands(&self, new_b: Option<Operand>, a: Operand) -> Self {
        //shoulda made this a stuct cause of shit like this :/
        match *self {
            Opcode::SET(ref b,_) => {
                Opcode::SET(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::ADD(ref b,_) => {
                Opcode::SET(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::SUB(ref b,_) => {
                Opcode::SET(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::MUL(ref b,_) => {
                Opcode::SET(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::MLI(ref b,_) => {
                Opcode::SET(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::DIV(ref b,_) => {
                Opcode::DIV(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::DVI(ref b,_) => {
                Opcode::DVI(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::MOD(ref b,_) => {
                Opcode::MOD(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::MDI(ref b,_) => {
                Opcode::MDI(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::AND(ref b,_) => {
                Opcode::AND(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::BOR(ref b,_) => {
                Opcode::BOR(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::XOR(ref b,_) => {
                Opcode::XOR(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::SHR(ref b,_) => {
                Opcode::SHR(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::ASR(ref b,_) => {
                Opcode::ASR(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::SHL(ref b,_) => {
                Opcode::SHL(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::IFB(ref b,_) => {
                Opcode::IFB(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::IFC(ref b,_) => {
                Opcode::IFC(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::IFE(ref b,_) => {
                Opcode::IFE(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::IFN(ref b,_) => {
                Opcode::IFN(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::IFG(ref b,_) => {
                Opcode::IFG(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::IFA(ref b,_) => {
                Opcode::IFA(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::IFL(ref b,_) => {
                Opcode::IFL(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::IFU(ref b,_) => {
                Opcode::IFU(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::ADX(ref b,_) => {
                Opcode::ADX(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::SBX(ref b,_) => {
                Opcode::SBX(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::STI(ref b,_) => {
                Opcode::STI(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::STD(ref b,_) => {
                Opcode::STD(new_b.unwrap_or(b.clone()), a)
            },
            Opcode::JSR(_) => {
                Opcode::JSR(a)
            },
            Opcode::INT(_) => {
                Opcode::INT(a)
            },
            Opcode::IAG(_) => {
                Opcode::IAG(a)
            },
            Opcode::IAS(_) => {
                Opcode::IAS(a)
            },
            Opcode::RFI(_) => {
                Opcode::RFI(a)
            },
            Opcode::IAQ(_) => {
                Opcode::IAQ(a)
            },
            Opcode::HWN(_) => {
                Opcode::HWN(a)
            },
            Opcode::HWQ(_) => {
                Opcode::HWQ(a)
            },
            Opcode::HWI(_) => {
                Opcode::HWI(a)
            }
        }
    }

    pub fn num_operands(&self) -> usize {
        match *self {
            Opcode::SET(_, _) |
            Opcode::ADD(_, _) |
            Opcode::SUB(_, _) |
            Opcode::MUL(_, _) |
            Opcode::MLI(_, _) |
            Opcode::DIV(_, _) |
            Opcode::DVI(_, _) |
            Opcode::MOD(_, _) |
            Opcode::MDI(_, _) |
            Opcode::AND(_, _) |
            Opcode::BOR(_, _) |
            Opcode::XOR(_, _) |
            Opcode::SHR(_, _) |
            Opcode::ASR(_, _) |
            Opcode::SHL(_, _) |
            Opcode::IFB(_, _) |
            Opcode::IFC(_, _) |
            Opcode::IFE(_, _) |
            Opcode::IFN(_, _) |
            Opcode::IFG(_, _) |
            Opcode::IFA(_, _) |
            Opcode::IFL(_, _) |
            Opcode::IFU(_, _) |
            Opcode::ADX(_, _) |
            Opcode::SBX(_, _) |
            Opcode::STI(_, _) |
            Opcode::STD(_, _) => { 2 },
            Opcode::JSR(_) |
            Opcode::INT(_) |
            Opcode::IAG(_) |
            Opcode::IAS(_) |
            Opcode::RFI(_) |
            Opcode::IAQ(_) |
            Opcode::HWN(_) |
            Opcode::HWQ(_) |
            Opcode::HWI(_) => { 1 }
        }
    }

    // returns a tuple containing references to operand A and optional operand B if present
    pub fn get_operands(&self) -> (&Operand, Option<&Operand>) {
        match *self {
            Opcode::SET(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::ADD(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::SUB(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::MUL(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::MLI(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::DIV(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::DVI(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::MOD(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::MDI(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::AND(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::BOR(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::XOR(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::SHR(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::ASR(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::SHL(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::IFB(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::IFC(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::IFE(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::IFN(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::IFG(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::IFA(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::IFL(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::IFU(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::ADX(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::SBX(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::STI(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::STD(ref b, ref a) => {
                (a, Some(b))
            },
            Opcode::JSR(ref a) => {
                (a, None)
            },
            Opcode::INT(ref a) => {
                (a, None)
            },
            Opcode::IAG(ref a) => {
                (a, None)
            },
            Opcode::IAS(ref a) => {
                (a, None)
            },
            Opcode::RFI(ref a) => {
                (a, None)
            },
            Opcode::IAQ(ref a) => {
                (a, None)
            },
            Opcode::HWN(ref a) => {
                (a, None)
            },
            Opcode::HWQ(ref a) => {
                (a, None)
            },
            Opcode::HWI(ref a) => {
                (a, None)
            }
        }
    }
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
