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

    rust_os_playground::init();

    // Invoke a breakpoint exception:
    // x86_64::instructions::interrupts::int3();

    // Trigger a page fault:
    // We use unsafe to write to the invalid address 0xdeadbeef.
    // The virtual address is not mapped to a physical address in
    // the page tables, so a page fault occurs. We haven’t registered
    // a page fault handler in our IDT, so a double fault occurs.
    unsafe {
        *(0xDEADBEEF as *mut u8) = 42;
    };

    #[cfg(test)]
    test_main();

    println!("we didn't crash on the interrupt!");

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
