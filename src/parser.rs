use nom::{IResult,digit,hex_digit,line_ending,multispace, ErrorKind, Err};
use nom::IResult::*;
use std::str;
use std::str::FromStr;
use std::num::ParseIntError;

#[derive(Debug,Clone,PartialEq)]
enum ParseError {
    LiteralExceedsSpace, //can't store literal in 16 bits
    InvalidHex(ParseIntError),
    InvalidDec(ParseIntError),
    UnknownInstruction,
    MissingComma,
    ItTakesTwo, //instruction takes 2 operands
    SoloTime, //instruction takes 1 operand
}

#[derive(Debug,Clone,PartialEq)]
struct ParseNumber<'a> {
    pub neg: bool,
    pub digit: &'a[u8],
    pub radix: u32,
}

#[derive(Debug,Clone,PartialEq)]
enum ParseOperand<'a> {
    Literal(u16),
    Label(&'a[u8]),
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
}

named!(comment,
    preceded!(tag!(";"),
        take_until_and_consume!("\n")));

named!(noise,
    chain!(
        many0!(alt!(multispace | comment)),
        || {&b""[..]}));

fn is_noise(c: &u8) -> bool {
    match *c {
        b' ' => true,
        b'\t' => true,
        b'\n' => true,
        b'\r' => true,
        b';' => true,
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
    map_res!(chain!(
        fix_error!(ParseError, tag!("#")) ~
        h: map_res!(take_till!(is_noise), str::from_utf8),
        || {
            h
        }
    ), from_hex_str)
);

named!(dec<&[u8], u16, ParseError>,
    map_res!(chain!(
        n: peek!(fix_error!(ParseError, tag!("-")))? ~
        i: map_res!(take_till!(is_noise), str::from_utf8),
        || {
            i
        }
    ), from_dec_str)
);

named!(literal<&[u8], u16, ParseError>,
       alt!(hex | dec));

named!(operand<&[u8], ParseOperand, ParseError>,
    alt!(
        literal => { |l| ParseOperand::Literal(l) }
    )
);

named!(label_def(&[u8]) -> &[u8],
    chain!(
        tag!(":") ~
        l: take_till!(is_noise),
        || { l }));

named!(opcode_double<&[u8], ParseOpcode, ParseError>,
    chain!(
        noise? ~
        o: add_error!(ErrorKind::Custom(ParseError::UnknownInstruction), fix_error!(ParseError, alt!(tag!("SET") | tag!("ADD") | tag!("SUB") |
                tag!("MUL") | tag!("DIV") | tag!("MLI") |
                tag!("DVI") | tag!("MOD") | tag!("MDI") |
                tag!("AND") | tag!("BOR") | tag!("XOR") |
                tag!("SHR") | tag!("ASR") | tag!("SHL") |
                tag!("IFB") | tag!("IFC") | tag!("IFE") |
                tag!("IFN") | tag!("IFG") | tag!("IFA") |
                tag!("IFL") | tag!("IFU") | tag!("ADX") |
                tag!("SBX") | tag!("STI") | tag!("STD")))) ~
        noise? ~
        a: operand ~
        noise? ~
        add_error!(ErrorKind::Custom(ParseError::MissingComma), fix_error!(ParseError,tag!(","))) ~
        noise? ~
        b: operand ~
        noise?,
        || {
            ParseOpcode{op: o, a: a, b: Some(b)}
        }));

named!(opcode_single<&[u8], ParseOpcode, ParseError>,
    chain!(
        noise? ~
        o: add_error!(ErrorKind::Custom(ParseError::UnknownInstruction), fix_error!(ParseError, alt!(tag!("JSR") | tag!("INT") | tag!("IAG") |
                     tag!("IAS") | tag!("RFI") | tag!("IAQ") |
                     tag!("HWN") | tag!("HWQ") | tag!("HWI")))) ~
        noise? ~
        a: operand ~
        noise?,
        || {
            ParseOpcode{op: o, a: a, b: None}
        }));

named!(opcode<&[u8], ParseOpcode, ParseError>,
    alt!(opcode_single | opcode_double));

#[test]
fn test_literal() {
    assert_eq!(IResult::Done(&b""[..], 0xFFFF), literal(b"#FFFF"));
    assert_eq!(IResult::Done(&b""[..], 255), literal(b"255"));
    assert_eq!(IResult::Done(&b""[..], -255i16 as u16), literal(b"-255"));
    assert_eq!(IResult::Done(&b""[..], -255i16 as u16), literal(b"-255.0"));
/*    assert_eq!(IResult::Done(&b"\n\tSTI"[..], ParseNumber{neg: false, digit: &b"FFFF"[..], radix: 16}),  literal(b"#FFFF\n\tSTI"));
    assert_eq!(IResult::Done(&b" "[..], ParseNumber{neg: false, digit: &b"255"[..], radix: 10}),  literal(b"255 "));
    assert_eq!(IResult::Done(&b" "[..], ParseNumber{neg: true, digit: &b"255"[..], radix: 10}),  literal(b"-255 "));
    assert_eq!(IResult::Done(&b"AND #FFFF -233"[..],
                             ParseOpcode {
                                 op: &b"JSR"[..],
                                 a: ParseOperand::Literal(ParseNumber { neg: false, digit: &b"AB2F"[..], radix: 16}),
                                 b: None}), opcode(b"\tJSR;bullshit layout\n#AB2F ;this is a comment\n\tAND #FFFF -233"));
    ::nom::print_error(&b"BSI #FFFF\n"[..], opcode_single(&b"BSI #FFFF\n"[..]));
    assert_eq!(Error(Err::NodePosition(ErrorKind::Custom(ParseError::UnknownInstruction), &b"BSI #FFFF\n"[..], Box::new(Err::Position(ErrorKind::Alt, &b"BSI #FFFF\n"[..])))), opcode_single(&b"BSI #FFFF\n"[..]));*/
}
