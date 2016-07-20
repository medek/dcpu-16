use std::fmt::{Debug, Formatter, Error};
use super::super::virtual_machine::VMExposed;
#[derive(Debug)]
pub struct HardwareInfo {
    pub manufacturer:u32,
    pub model:u32,
    pub version:u16
}

pub trait Hardware {
    fn info(&self) -> &HardwareInfo;
    fn hardware_interrupt(&mut self, &mut VMExposed) -> usize;
    fn update(&mut self, &mut VMExposed);
}

impl Debug for Hardware {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        fmt.write_str("TODO!")
    }
}
