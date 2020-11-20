use std::fmt::{Display, Debug, Formatter, Error};
use super::super::virtual_machine::VMExposed;

#[derive(Debug)]
pub struct HardwareInfo {
    pub manufacturer:u32,
    pub model:u32,
    pub version:u16
}

impl Display for HardwareInfo {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
            f.write_fmt(format_args!("Manufacturer ID: {:04x}, Model: {:04x}, Version: {:02x}",
                    self.manufacturer, self.model, self.version))
    }
}

pub trait Hardware {
    fn info(&self) -> &HardwareInfo;
    fn hardware_interrupt(&mut self, &mut VMExposed) -> usize;
    fn update(&mut self, &mut VMExposed);
    fn debug_dump_state(&self, fmt: &mut Formatter) -> Result<(), Error>;
}

impl Display for dyn Hardware {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        fmt.write_fmt(format_args!("{}", self.info()))
    }
}

impl Debug for dyn Hardware {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        fmt.write_fmt(format_args!("{}", *self.info()))?;
        self.debug_dump_state(fmt)
    }
}
