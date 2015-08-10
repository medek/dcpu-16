extern crate dcpu16;
use dcpu16::VirtualMachine;
use dcpu16::hardware::Clock;

const disasm:[u16;27] = [0x7c01, 0x0030, 0x7fc1, 0x0020, 0x1000,
                            0x7803, 0x1000, 0xc413, 0x7f81, 0x0019,
                            0xacc1, 0x7c01, 0x2000, 0x22c1, 0x2000,
                            0x88c3, 0x84d3, 0xbb81, 0x9461, 0x7c20,
                            0x0017, 0x7f81, 0x0019, 0x946f, 0x6381,
                            0xeb81, 0x0000];

fn main() {
    let mut vm = VirtualMachine::new()
        .attach_hardware(Clock::new())
        .load_program(&disasm.to_vec(), 0);

    vm.step();
    vm.step();
    println!("{:?}", vm);
}

