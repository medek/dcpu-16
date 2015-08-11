use super::super::virtual_machine::{VirtualMachine, Register};
use self::super::core::{Hardware, HardwareInfo};
use time::{Timespec, precise_time_s};

#[derive(Debug)]
pub struct RealtimeClock {
    hw_info: HardwareInfo,
    clock_rate: u16,
    last_time: f64,
    last_interrupt: f64,
    interrupt: u16,
}

pub struct VirtualClock {
    hw_info: HardwareInfo,
    ticks: u16,
    clock_rate: u16,
    last_cycles: usize,
    interrupt: u16,
}

impl VirtualClock {
    pub fn new() -> VirtualClock {
        VirtualClock { hw_info: HardwareInfo {
                manufacturer: 0x904b3115,
                model: 0x12d0b402,
                version: 0x0001
            },
            ticks: 0,
            clock_rate: 0,
            last_cycles: 0,
            interrupt: 0
        }
    }
}

impl RealtimeClock {
    pub fn new() -> RealtimeClock {
        RealtimeClock { hw_info: HardwareInfo {
                manufacturer: 0x904b3115,
                model: 0x12d0b402,
                version: 0x0001
            },
            clock_rate: 0,
            last_time: 0.0,
            last_interrupt: 0.0,
            interrupt: 0
        }
    }
}

impl<'a> Hardware for RealtimeClock {
    fn info(&self) -> &HardwareInfo {
        &self.hw_info
    }

    fn hardware_interrupt(&mut self, a: u16, registers: &mut [u16], ram: &mut [u16]) -> usize {
        match a {
            0x0 => {
                self.clock_rate = registers[Register::A as usize];
                self.last_time = precise_time_s();
            },
            0x1 => {
                registers[Register::C as usize] =
                    ((precise_time_s() - self.last_time) *
                     (60 as f64 / self.clock_rate as f64)) as u16;
            },
            0x2 => self.interrupt = registers[Register::B as usize],
            _ => return 0
        }
        0
    }

    fn update(&mut self, vm: &mut VirtualMachine) {
        if self.clock_rate != 0 {
            if self.interrupt != 0 {
                if precise_time_s() - self.last_interrupt > (60 as f64 / self.clock_rate as f64) {
                    vm.interrupt(self.interrupt);
                    self.last_interrupt = precise_time_s();
                }
            }
        }
    }
}
