#![no_std] // Do not link rust standard library.
#![no_main] // Disable all rust-level entrypoints.
#![feature(const_mut_refs)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

mod serial;
mod vga_buffer;

/// This function is called on panic.
/// This function should never return, so it is marked as
/// a diverging function by returning the “never” type !
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCodeCode::Success);
    loop {}
}

// Don't mangle function name (_start) - this is the entry point since the
// linker looks for a function named `_start` by default.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    use core::fmt::Write;

    vga_buffer::WRITER.lock().write_str("Hello again!").unwrap();
    write!(
        vga_buffer::WRITER.lock(),
        ", some numbers: {}, {}",
        42,
        1.337
    )
    .unwrap();

    println!();
    println!("Hello World{}", "!");
    println!();

    #[cfg(test)]
    test_main();

    loop {}
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

// Our runner just prints a short debug message and then calls each test function
// in the list. The argument type &[&dyn Fn()] is a slice of trait object references
// of the Fn() trait. It is basically a list of references to types that can be
// called like a function. Since the function is useless for non-test runs, we use
// the #[cfg(test)] attribute to include it only for tests.
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCodeCode::Success);
}

#[test_case]
fn trivial_assertion() {
    serial_print!("trivial assertion...");
    assert_eq!(1, 1);
    serial_println!("[ok]");
}
