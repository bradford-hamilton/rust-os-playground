#![no_std]  // Do not link rust standard library.
#![no_main] // Disable all rust-level entrypoints.
#![feature(const_mut_refs)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

mod vga_buffer;

/// This function is called on panic.
/// This function should never return, so it is marked as
/// a diverging function by returning the “never” type !
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop{}
}

// Don't mangle function name (_start) - this is the entry point since the
// linker looks for a function named `_start` by default.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    use core::fmt::Write;

    vga_buffer::WRITER.lock().write_str("Hello again!").unwrap();
    write!(vga_buffer::WRITER.lock(), ", some numbers: {}, {}", 42, 1.337).unwrap();

    println!();
    println!("Hello World{}", "!");
    println!();

    #[cfg(test)]
    test_main();

    loop{}
}

// Our runner just prints a short debug message and then calls each test function
// in the list. The argument type &[&dyn Fn()] is a slice of trait object references
// of the Fn() trait. It is basically a list of references to types that can be
// called like a function. Since the function is useless for non-test runs, we use
// the #[cfg(test)] attribute to include it only for tests.
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion...");
    assert_eq!(1, 1);
    println!("[ok]");
}
