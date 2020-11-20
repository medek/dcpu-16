use super::super::virtual_machine::{VMExposed, Register};
use self::super::core::{Hardware, HardwareInfo};
use std::fmt::{Formatter, Error};

pub struct Clock {
    hw_info: HardwareInfo,
    clock_rate: u16,
    last_cycles: usize,
    interrupt: u16,
    last_interrupt: usize
}

impl Clock {
    pub fn new() -> Clock {
        Clock { hw_info: HardwareInfo {
                manufacturer: 0x904b3115,
                model: 0x12d0b402,
                version: 0x0001
            },
            clock_rate: 0,
            last_cycles: 0,
            interrupt: 0,
            last_interrupt: 0
        }
    }
}

impl Hardware for Clock {
    fn info(&self) -> &HardwareInfo {
        &self.hw_info
    }

    fn hardware_interrupt(&mut self, vm: &mut VMExposed) -> usize {
        let (a, c) = vm.read_register(Register::A);
        let mut cycles = c;
        match a {
            0x0 => {
                let (cr, c) = vm.read_register(Register::B);
                self.clock_rate = cr;
                self.last_cycles = vm.get_cycles();
                cycles += c;
            },
            0x1 => {
                let ticks = (((vm.get_cycles() - self.last_cycles) as f64 *
                 (60 as f64 / self.clock_rate as f64))/vm.get_clock_rate() as f64) as u16;
                cycles += vm.write_register(Register::C, ticks);
            },
            0x2 => {
                let (i, c) = vm.read_register(Register::B);
                self.interrupt = i;
                cycles += c;
            },
            _ => return 0
        }
        cycles
    }

    fn update(&mut self, vm: &mut VMExposed) {
        if self.clock_rate != 0 {
            if self.interrupt != 0 {
                if (vm.get_cycles() - self.last_interrupt) as f64 > vm.get_clock_rate() as f64 / (60 as f64 / self.clock_rate as f64) {
                    self.last_interrupt = vm.get_cycles();
                    vm.interrupt(self.interrupt);
                }
            }
        }
    }

    fn debug_dump_state(&self, fmt: &mut Formatter) -> Result<(), Error> {
        fmt.write_fmt(
            format_args!("clock rate: {}, last cycles: {}, interrupt: {:02x}, last interrupt: {}",
                self.clock_rate, self.last_cycles, self.interrupt, self.last_interrupt))
    }
}
