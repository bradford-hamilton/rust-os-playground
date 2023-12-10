#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os_playground::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rust_os_playground::println;

// Don't mangle function name (_start) - this is the entry point since the
// linker looks for a function named `_start` by default.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main();

    loop {}
}

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
    rust_os_playground::test_panic_handler(info);
}
