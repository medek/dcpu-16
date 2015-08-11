use std::fmt::{Debug, Formatter, Error};
use super::super::virtual_machine::VirtualMachine;

#[derive(Debug)]
pub struct HardwareInfo {
    pub manufacturer:u32,
    pub model:u32,
    pub version:u16
}

pub trait Hardware {
    fn info(&self) -> &HardwareInfo;
    fn hardware_interrupt(&mut self, u16, &mut [u16], &mut [u16]) -> usize;
    fn update(&mut self, &mut VirtualMachine);
}

impl Debug for Hardware + Sized {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        fmt.write_str("TODO!")
    }
}
