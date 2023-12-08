#![no_std]  // Do not link rust standard library.
#![no_main] // Disable all rust-level entrypoints.

use core::panic::PanicInfo;

/// This function is called on panic.
/// This function should never return, so it is marked as
/// a diverging function by returning the “never” type !
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}

// the buffer is located at address 0xb8000 and that each character cell
// consists of an ASCII byte and a color byte.

static HELLO: &[u8] = b"Hello world!";

// Don't mangle function name (_start) - this is the entry point since the
// linker looks for a function named `_start` by default.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;    // Write the string byte
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb; // Corresponding color byte (light cyan)
        }
    }

    loop{}
}
