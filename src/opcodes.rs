/* use 32 bits for args so we can embed extra information and whatnot */
use std::fmt::{Display, Formatter, Error};
use virtual_machine::{VirtualMachine, Register};
use result::DcpuResult;
#[derive(Debug)]
pub enum Operand {
    Register(Register),
    RegisterDeRef(Register),
    RegisterPlusDeRef(Register, u16),
    Push,
    Pop,
    Peek,
    Pick(u16),
    Sp,
    Pc,
    Ex,
    LiteralDeRef(u16),
    Literal(u16),
}

#[derive(Debug)]
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

pub trait Disassemble {
    fn disassm(&self) -> DcpuResult<Vec<Opcode>>;
}

impl Display for Opcode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        fmt.write_str("TODO")
    }
}
