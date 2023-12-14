#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os_playground::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rust_os_playground::{memory::BootInfoFrameAllocator, println};

// Don't mangle function name (_start) - this is the entry point since
// the linker looks for a function named `_start` by default. Update:
// After adding the &'static BootInfo argument, to make sure that the
// entry point function always has the correct signature that the bootloader
// expects, the bootloader crate provides an entry_point macro that
// provides a type-checked way to define a Rust function as the entry
// point. Let’s rewrite our entry point function to use this macro:
entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    rust_os_playground::init();
    println!("Hello World{}", "!");

    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    #[cfg(test)]
    test_main();

    rust_os_playground::hlt_loop();
}

/// This function is called on panic.
/// This function should never return, so it is marked as
/// a diverging function by returning the “never” type !
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rust_os_playground::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os_playground::test_panic_handler(info);
}
