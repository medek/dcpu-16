use std::fmt::{Display, Formatter, Error};

#[derive(Debug)]
pub enum DcpuErrorKind {
    PushInAOp,
    PopInBOp,
    MissingNextWord,
    ReservedOpcode(u16),
    EmptyIterator,
    EmptyInstruction
}

#[derive(Debug)]
pub struct DcpuError {
    pub reason: DcpuErrorKind
}

pub type DcpuResult<T> = Result<T, DcpuError>;

impl Display for DcpuError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self.reason {
            DcpuErrorKind::PushInAOp =>
                fmt.write_str("Invalid A operand (cannot put push there)"),
            DcpuErrorKind::PopInBOp =>
                fmt.write_str("Invalid B operand (cannot put pop there)"),
            DcpuErrorKind::MissingNextWord =>
                fmt.write_str("Missing next word needed by opcode"),
            DcpuErrorKind::ReservedOpcode(op) =>
                fmt.write_fmt(format_args!("Reserved opcode found {:01x}", op)),
            DcpuErrorKind::EmptyIterator =>
                fmt.write_str("Someone passed in an empty iterator!"),
            DcpuErrorKind::EmptyInstruction =>
                fmt.write_str("0b0000000000000000 isn't a valid instruction!"),
        }
    }
}
