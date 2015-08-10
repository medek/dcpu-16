extern crate dcpu16;
use dcpu16::VirtualMachine;
use dcpu16::hardware::Clock;

fn main() {
    let mut vm = VirtualMachine::new();
    vm.attach_hardware(Clock::new());
}

