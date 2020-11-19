use opcodes::{Opcode, Operand}; 
use result::{DcpuError, DcpuResult, DcpuErrorKind};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub enum Intermediate {
    Opcode(Opcode),
    Label(String),
    Data(Vec<u8>),
    Reserve(usize)
}

#[derive(Debug)]
pub struct Block {
    intermediate: Vec<Intermediate>,
    symbols: BTreeMap<String, usize>, //symbols in the block and their index
}

impl Block {
    pub fn new() -> Block {
        Block {
            intermediate: Vec::new(),
            symbols: BTreeMap::new(),
        }
    }

    pub fn intermediate(mut self, inter: &mut Vec<Intermediate>) -> Self {
        for i in 0..inter.len() {
            match inter[i] {
                Intermediate::Label(ref s) => {
                        self.symbols.insert(s.clone(), i + self.intermediate.len());
                }
                _ => continue
            }
        }
        self.intermediate.append(inter);
        self
    }

    pub fn has_symbol(&self, s: &String) -> bool {
        self.symbols.contains_key(s)
    }
}
