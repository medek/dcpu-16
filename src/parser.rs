use pest::Parser;
#[allow(unused_imports)]
use pest::error::Error;
use pest::iterators::Pair;
use virtual_machine::Register as VMRegister;
use opcodes::{Opcode, Operand};
use thiserror::Error;

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("dcpu.pest");
#[derive(Parser)]
#[grammar = "dcpu.pest"]
struct DcpuParser;

#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    #[error("Parse Error: {}", .0)]
    PestError(#[from] pest::error::Error<Rule>),
    #[error("Literal {:04x} exceeds maximum size {:02x}", .0, u16::MAX)]
    ExceedsLiteralSize(u32),
    #[error("Failed to parse literal {}", .0)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Invalid deref on {}", .0)]
    InvalidDeref(String)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    LabelDef(String),
    Instruction(Opcode),
}

fn parse_int_literal(pair: Pair<Rule>) -> Result<u16, ParseError> {
    let num = i32::from_str_radix(pair.as_str(), 10)?;
    
    if num > u16::MAX.into() {
        return Err(ParseError::ExceedsLiteralSize(num as u32))
    }

    if i32::abs(num) > u16::MAX.into() { return Ok(0 as u16) }

    Ok(num as u16)
}

fn parse_hex_literal(pair: Pair<Rule>) -> Result<u16, ParseError> {
    let hb = pair.into_inner().as_str();
    let num = u32::from_str_radix(hb, 16)?;
    if num > u16::MAX.into() {
        return Err(ParseError::ExceedsLiteralSize(num))
    }

    Ok(num as u16)
}

fn parse_regsiter(s: String) -> Result<VMRegister, ParseError> {
    if let Some(reg) = VMRegister::from_str(&s) {
        return Ok(reg)
    }

    Err(ParseError::InvalidDeref(s))
}

fn emit_register(pair: Pair<Rule>) -> Result<Operand, ParseError> {

    if let Ok(reg) = parse_regsiter(String::from(pair.as_str())) {
        return Ok(Operand::Register(reg))
    }

    let mut inner = pair.into_inner();
    match inner.next().unwrap().as_rule() {
        Rule::sp => {
            return Ok(Operand::Sp)
        },
        Rule::ex => {
            return Ok(Operand::Ex)
        },
        Rule::pc => {
            return Ok(Operand::Pc)
        },
        unknown_term => panic!("Unknown term encountered in {}@{}:{}: {:#?}", file!(), line!(), column!(), unknown_term)
    }
}

fn emit_int_literal(pair: Pair<Rule>) -> Result<Operand, ParseError> {
    let num = parse_int_literal(pair)?;

    Ok(Operand::Literal(num as u16))
}

fn emit_literal_deref(pair: Pair<Rule>) -> Result<Operand, ParseError> {
    match pair.as_rule() {
        Rule::int_literal => {
            Ok(Operand::LiteralDeref(parse_int_literal(pair)?))
        },
        Rule::hex_literal => {
            Ok(Operand::LiteralDeref(parse_hex_literal(pair)?))
        },
        Rule::ident => {
            Ok(Operand::LabelDeref(String::from(pair.as_str())))
        },
        unknown_term => panic!("Unknown term encountered in {}@{}:{}: {:#?}", file!(), line!(), column!(), unknown_term)
    }
}

fn emit_hex_literal(pair: Pair<Rule>) -> Result<Operand, ParseError> {
    let num = parse_hex_literal(pair)?;

    Ok(Operand::Literal(num as u16))
}

fn emit_register_deref(pair: Pair<Rule>) -> Result<Operand, ParseError> {
    let reg_str = pair.as_str().to_uppercase();
    if let Some(reg) = VMRegister::from_str(&reg_str) {
        return Ok(Operand::RegisterDeref(reg))
    }

    match pair.as_rule() {
        Rule::pc => {
            Err(ParseError::InvalidDeref(pair.as_str().to_string()))
        },
        Rule::sp => {
            Ok(Operand::Peek)
        },
        Rule::ex => {
            Err(ParseError::InvalidDeref(pair.as_str().to_string()))
        },
        unknown_term => panic!("Unknown term encountered in {}@{}:{}: {:#?}", file!(), line!(), column!(), unknown_term)
    }
}

fn emit_register_plus_deref(pair: Pair<Rule>) -> Result<Operand, ParseError> {
    let mut inner = pair.into_inner();
    let lhs = parse_regsiter(inner.next().unwrap().as_str().to_string())?;
    if let Some(rhs) = inner.next() {
        match rhs.as_rule() {
            Rule::ident => {
                return Ok(Operand::RegisterPlusLabelDeref(lhs, String::from(rhs.as_str())))
            },
            Rule::int_literal => {
                return Ok(Operand::RegisterPlusDeref(lhs, parse_int_literal(rhs)?))
            },
            Rule::hex_literal => {
                return Ok(Operand::RegisterPlusDeref(lhs, parse_hex_literal(rhs)?))
            },
            unknown_term => panic!("Unknown term encountered in {}@{}:{}: {:#?}", file!(), line!(), column!(), unknown_term)
        }
    }
    unreachable!()
}

fn emit_ident_plus_deref(pair: Pair<Rule>) -> Result<Operand, ParseError> {
    let mut inner = pair.into_inner();
    let lhs = inner.next().unwrap().as_str().to_string();

    if let Some(rhs) = inner.next() {
        match rhs.as_rule() {
            Rule::ident => {
                return Ok(Operand::LabelPlusLabelDeref(lhs, String::from(rhs.as_str())))
            },
            Rule::int_literal => {
                return Ok(Operand::LabelPlusDeref(lhs, parse_int_literal(rhs)?))
            },
            Rule::hex_literal => {
                return Ok(Operand::LabelPlusDeref(lhs, parse_hex_literal(rhs)?))
            },
            unknown_term => panic!("Unknown term encountered in {}@{}:{}: {:#?}", file!(), line!(), column!(), unknown_term)
        }
    }
    unreachable!()
}

fn emit_deref(pair: Pair<Rule>) -> Result<Operand, ParseError> {
    match pair.as_rule() {
        Rule::register => {
            return Ok(emit_register_deref(pair.into_inner().next().unwrap())?)
        },
        Rule::register_plus_deref => {
            return Ok(emit_register_plus_deref(pair)?)
        },
        Rule::literal_deref => {
            return Ok(emit_literal_deref(pair.into_inner().next().unwrap())?)
        },
        Rule::ident_plus_deref => {
            return Ok(emit_ident_plus_deref(pair)?)
        },
        unknown_term => panic!("Unknown term encountered in {}@{}:{}: {:#?}", file!(), line!(), column!(), unknown_term)
    }
}

fn emit_operand(pair: Pair<Rule>) -> Result<Operand, ParseError> {
    match pair.as_rule() {
        Rule::operand => {
            let inner = pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::register => {
                     emit_register(inner)
                },
                Rule::deref => {
                     emit_deref(inner.into_inner().next().unwrap())
                },
                Rule::int_literal => {
                     emit_int_literal(inner)
                },
                Rule::hex_literal => {
                     emit_hex_literal(inner)
                },
                Rule::ident => {
                     Ok(Operand::Label(String::from(inner.as_str())))
                },
                unknown_term => panic!("Unknown term encountered in {}@{}:{}: {:#?}", file!(), line!(), column!(), unknown_term)
            }
        },
        Rule::push_operand => {
            Ok(Operand::Push)
        },
        Rule::pop_operand => {
            Ok(Operand::Pop)
        },
        unknown_term => panic!("Unknown term encountered in {}@{}:{}: {:#?}", file!(), line!(), column!(), unknown_term)
    }
}

fn emit_opcode_single(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner();
    let op = inner.next().unwrap();
    let operand = inner.next().unwrap();
    match op.as_rule() {
        Rule::op_jsr => {
            Ok(Statement::Instruction(Opcode::JSR(emit_operand(operand)?)))
        },
        Rule::op_int => {
            Ok(Statement::Instruction(Opcode::INT(emit_operand(operand)?)))
        },
        Rule::op_iag => {
            Ok(Statement::Instruction(Opcode::IAG(emit_operand(operand)?)))
        },
        Rule::op_rfi => {
            Ok(Statement::Instruction(Opcode::RFI(emit_operand(operand)?)))
        },
        Rule::op_iaq => {
            Ok(Statement::Instruction(Opcode::IAG(emit_operand(operand)?)))
        },
        Rule::op_hwn => {
            Ok(Statement::Instruction(Opcode::HWN(emit_operand(operand)?)))
        },
        Rule::op_hwq => {
            Ok(Statement::Instruction(Opcode::HWQ(emit_operand(operand)?)))
        },
        Rule::op_hwi => {
            Ok(Statement::Instruction(Opcode::HWI(emit_operand(operand)?)))
        },
        unknown_term => panic!("Unknown term encountered in {}@{}:{}: {:#?}", file!(), line!(), column!(), unknown_term)
    }
}

fn emit_opcode_double(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner();
    let op = inner.next().unwrap();
    let lhs = inner.next().unwrap();
    let rhs = inner.next().unwrap();
    
    match op.as_rule() {
        Rule::op_set => {
            Ok(Statement::Instruction(Opcode::SET(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_add => {
            Ok(Statement::Instruction(Opcode::ADD(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_sub => {
            Ok(Statement::Instruction(Opcode::SUB(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_mul => {
            Ok(Statement::Instruction(Opcode::MUL(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_mli => {
            Ok(Statement::Instruction(Opcode::MLI(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_div => {
            Ok(Statement::Instruction(Opcode::DIV(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_dvi => {
            Ok(Statement::Instruction(Opcode::DVI(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_mod => {
            Ok(Statement::Instruction(Opcode::MOD(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_mdi => {
            Ok(Statement::Instruction(Opcode::MDI(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_and => {
            Ok(Statement::Instruction(Opcode::AND(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_bor => {
            Ok(Statement::Instruction(Opcode::BOR(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_xor => {
            Ok(Statement::Instruction(Opcode::XOR(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_shr => {
            Ok(Statement::Instruction(Opcode::SHR(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_asr => {
            Ok(Statement::Instruction(Opcode::ASR(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_shl => {
            Ok(Statement::Instruction(Opcode::SHL(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_ifb => {
            Ok(Statement::Instruction(Opcode::IFB(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_ifc => {
            Ok(Statement::Instruction(Opcode::IFC(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_ife => {
            Ok(Statement::Instruction(Opcode::IFE(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_ifn => {
            Ok(Statement::Instruction(Opcode::IFN(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_ifg => {
            Ok(Statement::Instruction(Opcode::IFG(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_ifa => {
            Ok(Statement::Instruction(Opcode::IFA(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_ifl => {
            Ok(Statement::Instruction(Opcode::IFL(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_ifu => {
            Ok(Statement::Instruction(Opcode::IFU(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_adx => {
            Ok(Statement::Instruction(Opcode::ADX(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_sbx => {
            Ok(Statement::Instruction(Opcode::SBX(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_sti => {
            Ok(Statement::Instruction(Opcode::STI(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        Rule::op_std => {
            Ok(Statement::Instruction(Opcode::STD(emit_operand(lhs)?, emit_operand(rhs)?)))
        },
        unknown_term => panic!("Unknown term encountered in {}@{}:{}: {:#?}", file!(), line!(), column!(), unknown_term)
    }
}

fn emit_label_def(pair: Pair<Rule>) -> Statement {
    //can only be label_def rule
    let ident = pair.into_inner();
    Statement::LabelDef(String::from(ident.as_str()))
}

pub fn parse(src: &str) -> Result<Vec<Statement>, ParseError> {
    let mut ret = vec![];
    let pairs = DcpuParser::parse(Rule::input, src)?;

    for pair in pairs {
        match pair.as_rule() {
            Rule::label_def => {
                ret.push(emit_label_def(pair));
            },
            Rule::opcode_double => {
                ret.push(emit_opcode_double(pair)?)
            },
            Rule::opcode_single => {
                ret.push(emit_opcode_single(pair)?)
            },
            Rule::EOI => { break; }
            unknown_term => panic!("Unknown term encountered in {}@{}:{}: {:#?}", file!(), line!(), column!(), unknown_term)
        }
    }

    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pest::*;
    #[test]
    fn int_literal() {
        parses_to! {
            parser: DcpuParser,
            input: "-123",
            rule: Rule::int_literal,
            tokens: [
                int_literal(0, 4)
            ]
        };
        parses_to! {
            parser: DcpuParser,
            input: "123",
            rule: Rule::int_literal,
            tokens: [
                int_literal(0, 3)
            ]
        };
        let mut pairs = DcpuParser::parse(Rule::int_literal, "-1234").unwrap();
        assert_eq!(emit_int_literal(pairs.peek().unwrap()), Ok(Operand::Literal(-1234i32 as u16)));

        pairs = DcpuParser::parse(Rule::int_literal, "65536").unwrap();
        assert_eq!(emit_int_literal(pairs.peek().unwrap()), Err(ParseError::ExceedsLiteralSize(65536)));
        
        pairs = DcpuParser::parse(Rule::int_literal, "12345").unwrap();
        assert_eq!(emit_int_literal(pairs.peek().unwrap()), Ok(Operand::Literal(12345u16)));
    }

    #[test]
    fn hex_literal() {
        parses_to! {
            parser: DcpuParser,
            input: "#ABCD",
            rule: Rule::hex_literal,
            tokens: [
                hex_literal(0, 5, [hex_body(1,5)])
            ]
        };
        parses_to! {
            parser: DcpuParser,
            input: "#FF11",
            rule: Rule::hex_literal,
            tokens: [
                hex_literal(0, 5, [hex_body(1,5)])
            ]
        };
        parses_to! {
            parser: DcpuParser,
            input: "0x1111",
            rule: Rule::hex_literal,
            tokens: [
                hex_literal(0, 6, [hex_body(2, 6)])
            ]
        };
        let mut pairs = DcpuParser::parse(Rule::hex_literal, "0x1234").unwrap();

        assert_eq!(emit_hex_literal(pairs.peek().unwrap()), Ok(Operand::Literal(0x1234 as u16)));

        pairs = DcpuParser::parse(Rule::hex_literal, "0x12345").unwrap();
        assert_eq!(emit_hex_literal(pairs.peek().unwrap()), Err(ParseError::ExceedsLiteralSize(0x12345)));
    }

    #[test]
    fn labels() {
        parses_to! {
            parser: DcpuParser,
            input: ":foo_bar1234",
            rule: Rule::label_def,
            tokens: [
                label_def(0, 12, [ident(1, 12)])
            ]
        };

        let mut pairs = DcpuParser::parse(Rule::label_def, ":this_is_a_test").unwrap();

        assert_eq!(emit_label_def(pairs.next().unwrap()), Statement::LabelDef("this_is_a_test".to_string()));
    }

    #[test]
    fn opcode_single() {
        parses_to! {
            parser: DcpuParser,
            input: "JSR #1234",
            rule: Rule::opcode_single,
            tokens: [
                opcode_single(0, 9, [
                    op_jsr(0, 3),
                    operand(4, 9, [
                        hex_literal(4, 9, [hex_body(5, 9)])
                    ])
                ])
            ]
        };
        parses_to! {
            parser: DcpuParser,
            input: "JSR 1234",
            rule: Rule::opcode_single,
            tokens: [
                opcode_single(0, 8, [
                       op_jsr(0, 3),
                    operand(4, 8, [
                        int_literal(4, 8)
                    ])
                ])
            ]
        };
        let mut pairs = DcpuParser::parse(Rule::input, "IAG [some_label]").unwrap();
        assert_eq!(emit_opcode_single(pairs.next().unwrap()), Ok(Statement::Instruction(Opcode::IAG(Operand::LabelDeref("some_label".to_string())))));
    }

    #[test]
    fn opcode_double() {
        parses_to! {
            parser: DcpuParser,
            input: "SET PUSH, [some_label]",
            rule: Rule::opcode_double,
            tokens: [
                opcode_double(0, 22, [
                    op_set(0, 3),
                    push_operand(4,8),
                    operand(10, 22, [deref(10, 22, [literal_deref(11,21,[ident(11, 21)])])])
                ])
            ]
        };
        
        parses_to! {
            parser: DcpuParser,
            input: "SET [some_label], #10",
            rule: Rule::opcode_double,
            tokens: [
                opcode_double(0, 21, [
                    op_set(0, 3),
                    operand(4, 16, [deref(4, 16, [literal_deref(5,15,[ident(5, 15)])])]),
                    operand(18, 21, [hex_literal(18, 21, [hex_body(19, 21)])])
                ])
            ]
        };
        let mut pairs = DcpuParser::parse(Rule::input, "SET [some_label], #10").unwrap();
        assert_eq!(emit_opcode_double(pairs.next().unwrap()),
                Ok(Statement::Instruction(
                        Opcode::SET(
                            Operand::LabelDeref("some_label".to_string()),
                            Operand::Literal(0x10))
                        )
                    )
        );
    }
    #[test]
    fn operand() {
        parses_to! {
            parser: DcpuParser,
            input: "some_label",
            rule: Rule::operand,
            tokens: [
                operand(0, 10, [ident(0, 10)])
            ]
        };
        parses_to! {
            parser: DcpuParser,
            input: "I",
            rule: Rule::operand,
            tokens: [
                operand(0, 1, [register(0, 1)])
            ]
        };
        let mut pairs = DcpuParser::parse(Rule::operand, "A").unwrap();

        assert_eq!(emit_operand(pairs.next().unwrap()), Ok(Operand::Register(VMRegister::A)));

        pairs = DcpuParser::parse(Rule::operand, "pc").unwrap();
        assert_eq!(emit_operand(pairs.next().unwrap()), Ok(Operand::Pc));

        pairs = DcpuParser::parse(Rule::operand, "some_label").unwrap();
        assert_eq!(emit_operand(pairs.next().unwrap()), Ok(Operand::Label(String::from("some_label"))));
         
        pairs = DcpuParser::parse(Rule::operand, "0x1234").unwrap();
        assert_eq!(emit_operand(pairs.next().unwrap()), Ok(Operand::Literal(0x1234)));
        
        pairs = DcpuParser::parse(Rule::operand, "0x12345").unwrap();
        assert_eq!(emit_operand(pairs.next().unwrap()), Err(ParseError::ExceedsLiteralSize(0x12345)));
        
        pairs = DcpuParser::parse(Rule::operand, "[SP]").unwrap();
        assert_eq!(emit_operand(pairs.next().unwrap()), Ok(Operand::Peek));
        
        pairs = DcpuParser::parse(Rule::operand, "[0x1234]").unwrap();
        assert_eq!(emit_operand(pairs.next().unwrap()), Ok(Operand::LiteralDeref(0x1234)));

        pairs = DcpuParser::parse(Rule::operand, "[PC]").unwrap();
        assert_eq!(emit_operand(pairs.next().unwrap()), Err(ParseError::InvalidDeref("PC".to_string())));

        pairs = DcpuParser::parse(Rule::operand, "[A+0x12]").unwrap();
        assert_eq!(emit_operand(pairs.next().unwrap()), Ok(Operand::RegisterPlusDeref(VMRegister::A, 0x12)));
        
        pairs = DcpuParser::parse(Rule::operand, "[foobar+0x12]").unwrap();
        assert_eq!(emit_operand(pairs.next().unwrap()), Ok(Operand::LabelPlusDeref("foobar".to_string(), 0x12)));
        
        pairs = DcpuParser::parse(Rule::operand, "[A+foobar]").unwrap();
        assert_eq!(emit_operand(pairs.next().unwrap()),
                    Ok(Operand::RegisterPlusLabelDeref(VMRegister::A, "foobar".to_string())));
        pairs = DcpuParser::parse(Rule::operand, "[testing + foobar]").unwrap();
        assert_eq!(emit_operand(pairs.next().unwrap()),
                    Ok(Operand::LabelPlusLabelDeref("testing".to_string(), "foobar".to_string())));
    }

    #[test]
    fn statement() {
        parses_to! {
            parser: DcpuParser,
            input: "SET PC, some_label",
            rule: Rule::statement,
            tokens: [
                opcode_double(0, 18, [
                    op_set(0,3),
                    operand(4, 6, [register(4,6, [pc(4,6)])]),
                    operand(8, 18, [ident(8, 18)])
                ]),
            ]
        };
        parses_to! {
            parser: DcpuParser,
            input: ":some_label",
            rule: Rule::statement,
            tokens: [
                label_def(0, 11, [ident(1, 11)])
            ]
        };
        parses_to! {
            parser: DcpuParser,
            input: "IAG some_label",
            rule: Rule::statement,
            tokens: [
                opcode_single(0, 14, [
                    op_iag(0, 3),
                        operand(4, 14, [ident( 4,14)]),
                    ]),
                ]
        };
    }

    #[test]
    fn full() {
        parses_to! {
            parser: DcpuParser,
            input: ";testing\n:some_label SET PC, some_label ;another test\n;awdawdawd",
            rule: Rule::input,
            tokens: [
                label_def(9, 20, [ident(10, 20)]),
                opcode_double(21, 39, [
                    op_set(21, 24),
                    operand(25, 27, [register(25, 27, [pc(25,27)])]),
                    operand(29, 39, [ident(29, 39)])
                ])
            ]
        };
        #[cfg(debug_assertions)]
        const SIMPLE_ASM: &'static str = include_str!("../test/simple.asm");

        let statements = parse(SIMPLE_ASM).unwrap();
        assert_eq!(statements,
            vec![
            Statement::LabelDef("start".to_string()),
            Statement::Instruction(Opcode::SET(Operand::Register(VMRegister::A),
                                                Operand::Literal(0x30))),
            Statement::Instruction(Opcode::SET(Operand::Register(VMRegister::C),
                                                Operand::Literal(0x30))),
            Statement::Instruction(Opcode::SET(Operand::Register(VMRegister::B),
                                                Operand::Literal(123))),
            Statement::Instruction(Opcode::JSR(Operand::Label("start".to_string())))]);
    }
}
