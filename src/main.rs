#![no_std]  // Do not link rust standard library.
#![no_main] // Disable all rust-level entrypoints.
#![feature(const_mut_refs)]

use core::panic::PanicInfo;

mod vga_buffer;

/// This function is called on panic.
/// This function should never return, so it is marked as
/// a diverging function by returning the “never” type !
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}

// Don't mangle function name (_start) - this is the entry point since the
// linker looks for a function named `_start` by default.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    use core::fmt::Write;

    vga_buffer::WRITER.lock().write_str("Hello again!").unwrap();
    write!(vga_buffer::WRITER.lock(), ", some numbers: {}, {}", 42, 1.337).unwrap();

    loop{}
}
