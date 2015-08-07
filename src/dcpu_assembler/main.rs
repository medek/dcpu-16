#[macro_use]
extern crate dcpu16;
extern crate clap;

use dcpu16::{Opcode, Operand, Register, Assemble, Disassemble, VirtualMachine};
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
    let disasm:Vec<u16> = vec![0x7c01, 0x0030, 0x7fc1, 0x0020, 0x1000,
                                0x7803, 0x1000, 0xc413, 0x7f81, 0x0019,
                                0xacc1, 0x7c01, 0x2000, 0x22c1, 0x2000,
                                0x88c3, 0x84d3, 0xbb81, 0x9461, 0x7c20,
                                0x0017, 0x7f81, 0x0019, 0x946f, 0x6381,
                                0xeb81, 0x0000];
    println!("{:?}", disasm.disassm());
}
