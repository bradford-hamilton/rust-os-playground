// Like the main.rs, the lib.rs is a special file that is automatically recognized by cargo.
// The library is a separate compilation unit, so we need to specify the #![no_std] attribute again.
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod serial;
pub mod vga_buffer;

use core::panic::PanicInfo;

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

pub trait Testable {
    fn run(&self) -> ();
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
    exit_qemu(QemuExitCodeCode::Success);
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCodeCode {
    Success = 0x10,
    Failure = 0x11,
}

// The function creates a new Port at 0xf4, which is the iobase of the isa-debug-exit device.
// Then it writes the passed exit code to the port. We use u32 because we specified the iosize
// of the isa-debug-exit device as 4 bytes. Both operations are unsafe because writing to an
// I/O port can generally result in arbitrary behavior.
pub fn exit_qemu(exit_code: QemuExitCodeCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xF4);
        port.write(exit_code as u32)
    }
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCodeCode::Success);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
