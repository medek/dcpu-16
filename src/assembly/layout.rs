use opcodes::{Opcode, Operand};
use parser::Statement as ParseStatement;
use std::collections::{BTreeMap, VecDeque};
use thiserror::Error;
use assembly::{Assemble, AssemblyError};
use std::rc::Rc;

#[derive(Debug, Error)]
pub enum LayoutError {
    #[error("Symbol redefined: {}, @ {:x}, previously @ {:x}", .0, .1, .2)]
    RedefinedSymbol(String, usize, usize),
    #[error("Assembly failed: {}", .0)]
    AssemblyError(#[from]AssemblyError),
    #[error("Missing symbol: {}", .0)]
    MissingSymbol(String)
}

#[derive(Debug, PartialEq)]
enum LayoutProgress {
    Continue,
    Deferred,
}

#[derive(Debug,Clone)]
enum LayoutStatus {
    Block(Rc<Block>),
    DeferredStatement(Opcode)
}

#[derive(Debug,Clone,PartialEq)]
struct Block {
    code: Vec<u16>,
    symbols: BTreeMap<String, usize>
}


#[derive(Debug,Clone)]
pub struct Layout {
    curr: Block,
    layout: Vec<LayoutStatus>,
    symbol_pos: BTreeMap<String, usize>,
}

impl Block {
    pub fn new() -> Self {
        Block {
            code: Vec::new(),
            symbols: BTreeMap::new()
        }
    }

    /// if pos is None use last position in current block 
    pub fn define_label(&mut self, symbol: &String, pos: Option<usize>) -> Result<(), LayoutError> {
        let p = pos.unwrap_or(self.code.len());
        if let Some(ol_p) = self.symbols.get(symbol) {
            return Err(LayoutError::RedefinedSymbol(symbol.clone(), p, *ol_p))
        }

        self.symbols.insert(symbol.clone(), p);
        Ok(())
    }
    
    fn handle_label_operand(&self, operand: Operand) -> Option<Operand> {
        match operand {
            Operand::Label(ref s) => {
                if let Some(pos) = self.symbols.get(s) {
                    return Some(Operand::Literal(*pos as u16));
                }
            },
            Operand::LabelDeref(ref s) => {
                if let Some(pos) = self.symbols.get(s) {
                    return Some(Operand::Literal(*pos as u16));
                }
            },
            Operand::LabelPlusDeref(ref s, _) => {
                if let Some(pos) = self.symbols.get(s) {
                    return Some(Operand::Literal(*pos as u16));
                }
            },
            Operand::LabelPlusLabelDeref(ref s1, ref s2) => {
                let mut deferred = false;
                let mut p1:usize = 0;
                let mut p2:usize = 0;
                if let Some(pos) = self.symbols.get(s1) {
                    p1 = *pos;
                }
                else {
                    deferred = true;
                }

                if let Some(pos) = self.symbols.get(s2) {
                    p2 = *pos;
                }
                else {
                    deferred = true;
                }

                if !deferred {
                    return Some(Operand::Literal((p1 + p2) as u16))
                }
            },
            _ => return Some(operand.clone())
        }
        None
    }

    pub fn handle_statement(&mut self, op: &Opcode) -> Result<LayoutProgress, LayoutError> {
        let mut new_op:Opcode;
        let (a,b) = op.get_operands();
        let new_a = self.handle_label_operand(a.clone());
        if new_a.is_none() {
            return Ok(LayoutProgress::Deferred)
        }

        if b.is_some() {
            let new_b = self.handle_label_operand(b.unwrap().clone());
            if new_b.is_none() {
                return Ok(LayoutProgress::Deferred)
            }

            new_op = op.set_operands(new_b, new_a.unwrap());
            self.code.append(&mut new_op.assemble()?);
            return Ok(LayoutProgress::Continue)
        }
        else {
            new_op = op.set_operands(None, new_a.unwrap());
            self.code.append(&mut new_op.assemble()?);
            return Ok(LayoutProgress::Continue)
        }
    }

    pub fn size(&self) -> usize {
        self.code.len()
    }

    pub fn join(&mut self, other: &mut Block) -> Result<(), LayoutError> {
        let old_size = self.code.len();
        self.code.append(&mut other.code);

        for (k,v) in other.symbols.iter() {
            if let Some(p) = self.symbols.insert(k.clone(), v+old_size) {
                    return Err(LayoutError::RedefinedSymbol(k.clone(), v+old_size, p))
            }
        }
        other.symbols.clear();
        Ok(())
    }
}

impl Layout {
    pub fn new() -> Self {
        Layout {
            curr: Block::new(),
            layout: Vec::new(),
            symbol_pos: BTreeMap::new(),
        }
    }
    
    ///resolves all symbols and joins all completed blocks
    pub fn finalize(&mut self) -> Result<(), LayoutError> {
        unimplemented!()
    }

    fn resolve_symbol(&mut self) -> Result<(), LayoutError> {
        //.0 where the symbol is used, 0.1 where it is defined
        let mut stack:VecDeque<(usize, usize)> = VecDeque::new();

        for (i, item) in self.layout.enumerate() {
            match item {
                LayoutStatus::Block(_) => {continue},
                LayoutStatus::DeferredStatement(ref op) => {

                }
            }
        }
    }

    pub fn handle_statement(&mut self, stmt: &ParseStatement) -> Result<(), LayoutError> {
        match *stmt {
            ParseStatement::Instruction(ref op) => {
                match self.curr.handle_statement(op)? {
                    LayoutProgress::Deferred => {
                        self.layout.push(LayoutStatus::Block(Rc::new(self.curr.clone())));
                        self.layout.push(LayoutStatus::DeferredStatement(op.clone()));
                        self.curr = Block::new();
                    },
                    _ => { }
                }
                Ok(())
            },
            ParseStatement::LabelDef(ref s) => {
                self.curr.define_label(s, None)?;
                //since it'll end up in the current block and it's always pushed
                //first when deferring symbol resolution
                let pos = if self.layout.len() == 0 { 0 } else { self.layout.len() };
                self.symbol_pos.insert(s.clone(), pos);
                Ok(())
            }
        }
    }
}


