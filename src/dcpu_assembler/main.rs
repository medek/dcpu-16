#[macro_use]
extern crate dcpu16;
extern crate clap;

use dcpu16::{Opcode, Operand, Register, Assemble, Disassemble};
use clap::{App, Arg};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let matches = App::new("dcpu_assembler")
        .version(VERSION)
        .arg(Arg::with_name("input")
             .short("i")
             .long("input-file")
             .multiple(false)
             .takes_value(true))
        .arg(Arg::with_name("assemble")
             .short("a")
             .long("assemble")
             .takes_value(false)
             .conflicts_with("disassemble")
             .requires("input"))
        .arg(Arg::with_name("disassemble")
             .short("d")
             .long("disassemble")
             .takes_value(false)
             .conflicts_with("assemble")
             .requires("input"))
        .get_matches();
}
