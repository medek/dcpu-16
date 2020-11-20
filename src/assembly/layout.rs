use opcodes::{Opcode, Operand};
use parser::Statement as ParseStatement;
use std::collections::BTreeMap;
use std::rc::Rc;
use thiserror::Error;
use assembly::{Assemble, AssemblyError};

#[derive(Debug, Error)]
pub enum LayoutError {
    #[error("Symbol redefined: {}, @ {:x}, previously @ {:x}", .0, .1, .2)]
    RedefinedSymbol(String, usize, usize),
    #[error("Assembly failed: {}", .0)]
    AssemblyError(#[from]AssemblyError)
}

#[derive(Debug, PartialEq)]
pub enum LayoutStatus {
    Continue,
    Deferred(Opcode)
}

#[derive(Debug)]
pub enum LayoutProgress {
    Deferred(ParseStatement),
    Finalized(Block)
}

#[derive(Debug)]
pub struct Block {
    code: Vec<u16>, //complied instructions
    symbols: BTreeMap<String, usize>, //symbols in the block and their relative position to this block
    pos: Option<usize>, //None returned if there is deferred items proceeding block
}

#[derive(Debug)]
pub struct Layout {
    root: Vec<Rc<LayoutProgress>>,
    symbols: BTreeMap<String, Rc<Block>>
}

impl Block {
    pub fn new(pos: Option<usize>) -> Self {
        Block {
            code: vec![],
            symbols: BTreeMap::new(),
            pos: pos
        }
    }

    pub fn consume_statement(&mut self, stmt: ParseStatement) -> Result<LayoutStatus, LayoutError> {
        match stmt {
            ParseStatement::LabelDef(ref s) => {
                let p = if self.code.len() == 0 { 0 } else { self.code.len()-1 };
                if let Some(prev) = self.symbols.insert(s.to_string(), p) {
                    return Err(LayoutError::RedefinedSymbol(s.clone(), self.code.len(), prev))                        
                }
                Ok(LayoutStatus::Continue)
            },
            ParseStatement::Instruction(ref op) => {
                let mut asm = op.assemble()?;
                self.code.append(&mut asm);
                Ok(LayoutStatus::Continue)
            }
        }
    }
}

