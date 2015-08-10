use super::super::virtual_machine::{VirtualMachine, Register};
use self::super::core::{Hardware, HardwareInfo};
#[derive(Debug)]
pub struct Clock {
    hw_info: HardwareInfo,
    ticks: u16,
    clock_rate: u16
}

impl Clock {
    pub fn new() -> Clock {
        Clock { hw_info: HardwareInfo {
                manufacturer: 0x904b3115,
                model: 0x12d0b402,
                version: 0x0001
            },
            ticks: 0,
            clock_rate: 0,
        }
    }
}

impl<'r> Hardware for Clock {
    fn info(&self) -> &HardwareInfo {
        &self.hw_info
    }

    fn hardware_interrupt(&mut self, a: u16, vm: &mut VirtualMachine) -> usize {
        match a {
            0x0 => self.clock_rate = (vm.get_registers())[Register::A as usize],
            0x1 => unimplemented!(),
            0x2 => unimplemented!(),
            _ => return 0
        }
        0
    }

    fn update(&mut self, vm: &mut VirtualMachine) {
    }
}
