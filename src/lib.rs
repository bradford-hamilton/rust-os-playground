// Like the main.rs, the lib.rs is a special file that is automatically recognized by cargo.
// The library is a separate compilation unit, so we need to specify the #![no_std] attribute again.
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

pub mod gdt;
pub mod interrupts;
pub mod serial;
pub mod vga_buffer;

use core::panic::PanicInfo;

// Remember, this _start function is used when running cargo test --lib,
// since Rust tests the lib.rs completely independently of the main.rs
/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    loop {}
}

pub fn init() {
    interrupts::init_idt();
    gdt::init();
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

// Our runner just prints a short debug message and then calls each test function
// in the list. The argument type &[&dyn Fn()] is a slice of trait object references
// of the Fn() trait. It is basically a list of references to types that can be
// called like a function. Since the function is useless for non-test runs, we use
// the #[cfg(test)] attribute to include it only for tests.
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

// The function creates a new Port at 0xf4, which is the iobase of the isa-debug-exit device.
// Then it writes the passed exit code to the port. We use u32 because we specified the iosize
// of the isa-debug-exit device as 4 bytes. Both operations are unsafe because writing to an
// I/O port can generally result in arbitrary behavior.
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xF4);
        port.write(exit_code as u32)
    }
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Success);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
