use nom::{InputLength,IResult,alpha,digit,hex_digit,line_ending,multispace, ErrorKind, Err};
use nom::IResult::*;
use std::str;
use std::str::FromStr;
use std::num::ParseIntError;
use virtual_machine::Register as VMRegister;

#[derive(Debug,Clone,PartialEq)]
enum ParseError {
    LiteralExceedsSpace, //can't store literal in 16 bits
    InvalidHex(ParseIntError),
    InvalidDec(ParseIntError),
    UnknownInstruction,
    UnknownRegister,
    MissingComma,
    BadLabel,
    ItTakesTwo, //instruction takes 2 operands
}

#[derive(Debug,Clone,PartialEq)]
enum ParseOperand<'a> {
    Literal(u16),
    Label(&'a[u8]),
    Register(VMRegister),
}

#[derive(Debug,Clone,PartialEq)]
struct ParseOpcode<'a> {
    pub op: &'a[u8],
    pub a: ParseOperand<'a>,
    pub b: Option<ParseOperand<'a>>
}

#[derive(Debug,Clone,PartialEq)]
enum Token<'a> {
    Opcode(ParseOpcode<'a>),
    Label(&'a[u8]),
    Data(&'a[u8]),
}

named!(comment,
    preceded!(tag!(";"),
        take_until_and_consume!("\n")));

named!(noise,
    do_parse!(
        many0!(alt!(multispace | comment)) >>
        (&b""[..])));

fn is_noise(c: &u8) -> bool {
    match *c {
        b' ' => true,
        b'\t' => true,
        b'\n' => true,
        b'\r' => true,
        b';' => true,
        b',' => true,
        _ => false
    }
}

fn from_hex_str(s: &str) -> Result<u16, ParseError> {
    match u16::from_str_radix(s, 16) {
        Ok(h) => Ok(h),
        Err(e) => Err(ParseError::InvalidHex(e))
    }
}

fn from_dec_str(s: &str) -> Result<u16, ParseError> {
    match i32::from_str(s) {
        Ok(res) => {
            if res < -(u16::max_value() as i32) / 2  || res > u16::max_value() as i32 {
                return Err(ParseError::LiteralExceedsSpace);
            }

            Ok(res as u16)
        },
        Err(e) => Err(ParseError::InvalidDec(e)),
    }
}

named!(hex<&[u8], u16, ParseError>,
    map_res!(do_parse!(
        fix_error!(ParseError, tag!("#")) >>
        h: map_res!(take_till!(is_noise), str::from_utf8) >>
        (h)
    ), from_hex_str)
);

named!(dec<&[u8], u16, ParseError>,
    map_res!(do_parse!(
        i: map_res!(take_till!(is_noise), str::from_utf8) >>
        (i)
    ), from_dec_str)
);

named!(literal<&[u8], u16, ParseError>,
       alt!(hex | dec));

named!(register<&[u8], VMRegister, ParseError>,
    do_parse!(
        reg: add_error!(ErrorKind::Custom(ParseError::UnknownRegister),
                fix_error!(ParseError, alt!(tag!("A") | tag!("B") | tag!("C") | tag!("X") |
                                            tag!("Y") | tag!("Z") | tag!("I") | tag!("J")))) >>
        fix_error!(ParseError, noise) >>
        (VMRegister::from_str(reg).unwrap())
        ));

named!(label<&[u8], &[u8], ParseError>,
    do_parse!(
        start: fix_error!(ParseError, peek!(alpha)) >>
        label: fix_error!(ParseError, cond!(start.is_none(), is_a!("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_"))) >>
        (label.unwrap()
    ))
);

named!(operand<&[u8], ParseOperand, ParseError>,
    alt!(
        literal => { |l| ParseOperand::Literal(l) } |
        register => {|r| ParseOperand::Register(r) } |
        label => {|lbl| ParseOperand::Label(lbl) }
    )
);

named!(label_def(&[u8]) -> &[u8],
    do_parse!(
        tag!(":") >>
        l: take_till!(is_noise) >>
        (l)));

named!(opcode_double<&[u8], ParseOpcode, ParseError>,
    do_parse!(
        noise? >>
        o: add_error!(ErrorKind::Custom(ParseError::UnknownInstruction), fix_error!(ParseError, alt!(tag!("SET") | tag!("ADD") | tag!("SUB") |
                tag!("MUL") | tag!("DIV") | tag!("MLI") |
                tag!("DVI") | tag!("MOD") | tag!("MDI") |
                tag!("AND") | tag!("BOR") | tag!("XOR") |
                tag!("SHR") | tag!("ASR") | tag!("SHL") |
                tag!("IFB") | tag!("IFC") | tag!("IFE") |
                tag!("IFN") | tag!("IFG") | tag!("IFA") |
                tag!("IFL") | tag!("IFU") | tag!("ADX") |
                tag!("SBX") | tag!("STI") | tag!("STD")))) >>
        noise >>
        a: operand >>
        noise >>
        add_error!(ErrorKind::Custom(ParseError::MissingComma), fix_error!(ParseError,tag!(","))) >>
        noise >>
        b: add_error!(ErrorKind::Custom(ParseError::ItTakesTwo), operand) >>
        noise? >>
        (ParseOpcode{op: o, a: a, b: Some(b)})
    ));

named!(opcode_single<&[u8], ParseOpcode, ParseError>,
    do_parse!(
        noise? >>
        o: add_error!(ErrorKind::Custom(ParseError::UnknownInstruction), fix_error!(ParseError, alt!(tag!("JSR") | tag!("INT") | tag!("IAG") |
                     tag!("IAS") | tag!("RFI") | tag!("IAQ") |
                     tag!("HWN") | tag!("HWQ") | tag!("HWI")))) >>
        noise >>
        a: operand >>
        noise? >>
        (ParseOpcode{op: o, a: a, b: None})
        ));

named!(opcode<&[u8], ParseOpcode, ParseError>,
    alt!(opcode_double | opcode_single));

#[test]
fn tests() {
    assert_eq!(IResult::Done(&b""[..], 0xFFFF), literal(b"#FFFF"));
    assert_eq!(IResult::Done(&b""[..], 255), literal(b"255"));
    assert_eq!(IResult::Done(&b""[..], -255i16 as u16), literal(b"-255"));
    assert_eq!(IResult::Done(&b""[..], VMRegister::X), register(b"X"));
    assert_eq!(IResult::Done(&b""[..],
                             ParseOpcode {
                                 op: &b"IAQ"[..],
                                 a: ParseOperand::Literal(4082),
                                 b: None}), opcode_single(b"IAQ #FF2"));
    assert_eq!(IResult::Done(&b", -10"[..],
                             ParseOpcode {
                                 op: &b"IAQ"[..],
                                 a: ParseOperand::Literal(0xFFFF),
                                 b: None}), opcode_single(b"IAQ #FFFF, -10"));
    assert_eq!(IResult::Done(&b""[..],
                             ParseOpcode {
                                 op: &b"AND"[..],
                                 a: ParseOperand::Literal(0xFFFF),
                                 b: Some(ParseOperand::Literal(255))}), opcode_double(b"AND #FFFF, 255"));

    assert_eq!(IResult::Done(&b""[..],
                             ParseOpcode {
                                 op: &b"ADD"[..],
                                 a: ParseOperand::Register(VMRegister::X),
                                 b: Some(ParseOperand::Literal(-255i16 as u16))}), opcode_double(b"ADD X, -255"));

    assert_eq!(IResult::Done(&b""[..],
                             ParseOpcode {
                                 op: &b"SET"[..],
                                 a: ParseOperand::Label(&b"Xsome_label"[..]),
                                 b: Some(ParseOperand::Literal(-255i16 as u16))}), opcode_double(b"SET 2some_label, -255"));
}
