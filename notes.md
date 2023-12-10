- Need a rust executable that doesn't link the standard library (“freestanding” or “bare-metal”) that way we can run on bare metal.

- To write an operating system kernel, we need code that does not depend on any operating system features. This means that we can’t use threads, files, heap memory, the network, random numbers, standard output, or any other features requiring OS abstractions or specific hardware.

- On x86, there are two firmware standards: the “Basic Input/Output System“ (BIOS) and the newer “Unified Extensible Firmware Interface” (UEFI). The BIOS standard is old and outdated, but simple and well-supported on any x86 machine since the 1980s. UEFI, in contrast, is more modern and has much more features, but is more complex to set up (at least in my opinion).

- When you turn on a computer, it loads the BIOS from some special flash memory located on the motherboard. The BIOS runs self-test and initialization routines of the hardware, then it looks for bootable disks. If it finds one, control is transferred to its bootloader, which is a 512-byte portion of executable code stored at the disk’s beginning. Most bootloaders are larger than 512 bytes, so bootloaders are commonly split into a small first stage, which fits into 512 bytes, and a second stage, which is subsequently loaded by the first stage.

- The bootloader has to determine the location of the kernel image on the disk and load it into memory. It also needs to switch the CPU from the 16-bit real mode first to the 32-bit protected mode, and then to the 64-bit long mode, where 64-bit registers and the complete main memory are available. Its third job is to query certain information (such as a memory map) from the BIOS and pass it to the OS kernel.

- GNU GRUB is generally the most popular bootloader for Linux systems.

- Cargo supports different target systems through the --target parameter. The target is described by a so-called target triple, which describes the CPU architecture, the vendor, the operating system, and the ABI. For example, the x86_64-unknown-linux-gnu target triple describes a system with an x86_64 CPU, no clear vendor, and a Linux operating system with the GNU ABI. Rust supports many different target triples, including arm-linux-androideabi for Android or wasm32-unknown-unknown for WebAssembly.

- For our target system, however, we require some special configuration parameters (e.g. no underlying OS), so none of the existing target triples fits. Fortunately, Rust allows us to define our own target through a JSON file.

- The easiest way to print text to the screen early on is the VGA text buffer. It is a special memory area mapped to the VGA hardware that contains the contents displayed on screen. It normally consists of 25 lines that each contain 80 character cells. Each character cell displays an ASCII character with some foreground and background colors.

- To turn our compiled kernel into a bootable disk image, we need to link it with a bootloader.

- Instead of writing our own bootloader, which is a project on its own, we use the bootloader crate. This crate implements a basic BIOS bootloader without any C dependencies, just Rust and inline assembly. To use it for booting our kernel, we need to add a dependency on it:


- Adding the bootloader as a dependency is not enough to actually create a bootable disk image. The problem is that we need to link our kernel with the bootloader after compilation, but cargo has no support for post-build scripts. To solve this problem, we created a tool named bootimage that first compiles the kernel and bootloader, and then links them together to create a bootable disk image.

Thanks for this! :)

- How does it work?
    The bootimage tool performs the following steps behind the scenes:

    - It compiles our kernel to an ELF file.
    - It compiles the bootloader dependency as a standalone executable.
    - It links the bytes of the kernel ELF file to the bootloader.
    - When booted, the bootloader reads and parses the appended ELF file. It then maps the program segments to virtual addresses in the page tables, zeroes the .bss section, and sets up a stack. Finally, it reads the entry point address (our _start function) and jumps to it.

- qemu-system-x86_64 -drive format=raw,file=target/x86_64_custom_target/debug/bootimage-rust-os-playground.bin (now automated through cargo)

- The VGA text buffer is a two-dimensional array with typically 25 rows and 80 columns, which is directly rendered to the screen.
    
    Bit(s)	Value
    0-7	    ASCII code point
    8-11	Foreground color
    12-14	Background color
    15	    Blink

- The second byte defines how the character is displayed. The first four bits define the foreground color, the next three bits the background color, and the last bit whether the character should blink. The following colors are available:

    Number	Color	    Number + Bright Bit	  Bright Color
    0x0	    Black	    0x8	                  Dark Gray
    0x1	    Blue	    0x9	                  Light Blue
    0x2	    Green	    0xa	                  Light Green
    0x3	    Cyan	    0xb	                  Light Cyan
    0x4	    Red	        0xc	                  Light Red
    0x5	    Magenta	    0xd	                  Pink
    0x6	    Brown	    0xe	                  Yellow
    0x7	    Light Gray	0xf	                  White

- Bit 4 is the bright bit, which turns, for example, blue into light blue. For the background color, this bit is repurposed as the blink bit.

- The writer will always write to the last line and shift lines up when a line is full (or on \n). The column_position field keeps track of the current position in the last row. The current foreground and background colors are specified by color_code and a reference to the VGA buffer is stored in buffer. Note that we need an explicit lifetime here to tell the compiler how long the reference is valid. The 'static lifetime specifies that the reference is valid for the whole program run time (which is true for the VGA text buffer).

- Note that we only have a single unsafe block in our code, which is needed to create a Buffer reference pointing to 0xb8000. Afterwards, all operations are safe. Rust uses bounds checking for array accesses by default, so we can’t accidentally write outside the buffer. Thus, we encoded the required conditions in the type system and are able to provide a safe interface to the outside.

- Unfortunately, it’s a bit more complicated for no_std applications such as our kernel. The problem is that Rust’s test framework implicitly uses the built-in test library, which depends on the standard library. This means that we can’t use the default test framework for our #[no_std] kernel.

- Fortunately, Rust supports replacing the default test framework through the unstable custom_test_frameworks feature. This feature requires no external libraries and thus also works in #[no_std] environments.

- The disadvantage compared to the default test framework is that many advanced features, such as should_panic tests, are not available. Instead, it is up to the implementation to provide such features itself if needed. This is ideal for us since we have a very special execution environment where the default implementations of such advanced features probably wouldn’t work anyway. For example, the #[should_panic] attribute relies on stack unwinding to catch the panics, which we disabled for our kernel.

- When we run cargo test now, we see that it now succeeds (if it doesn’t, see the note below). However, we still see our “Hello World” instead of the message from our test_runner. The reason is that our _start function is still used as entry point. The custom test frameworks feature generates a main function that calls test_runner, but this function is ignored because we use the #[no_main] attribute and provide our own entry point.

- The convention for integration tests in Rust is to put them into a tests directory in the project root (i.e., next to the src directory). Both the default test framework and custom test frameworks will automatically pick up and execute all tests in that directory.

- All integration tests are their own executables and completely separate from our main.rs. This means that each test needs to define its own entry point function.

- cargo clippy --all-targets --all-features

- Explained how to set up a test framework for our Rust kernel. We used Rust’s custom test frameworks feature to implement support for a simple #[test_case] attribute in our bare-metal environment. Using the isa-debug-exit device of QEMU, our test runner can exit QEMU after running the tests and report the test status. To print error messages to the console instead of the VGA buffer, we created a basic driver for the serial port.

- Here is a short overview of the things that the x86-interrupt calling convention takes care of:
    - Retrieving the arguments: Most calling conventions expect that the arguments are passed in registers. This is not possible for exception handlers since we must not overwrite any register values before backing them up on the stack. Instead, the x86-interrupt calling convention is aware that the arguments already lie on the stack at a specific offset.
    - Returning using iretq: Since the interrupt stack frame completely differs from stack frames of normal function calls, we can’t return from handler functions through the normal ret instruction. So instead, the iretq instruction must be used.
    - Handling the error code: The error code, which is pushed for some exceptions, makes things much more complex. It changes the stack alignment (see the next point) and needs to be popped off the stack before returning. The x86-interrupt calling convention handles all that complexity. However, it doesn’t know which handler function is used for which exception, so it needs to deduce that information from the number of function arguments. That means the programmer is still responsible for using the correct function type for each exception. Luckily, the InterruptDescriptorTable type defined by the x86_64 crate ensures that the correct function types are used.
    - Aligning the stack: Some instructions (especially SSE instructions) require a 16-byte stack alignment. The CPU ensures this alignment whenever an exception occurs, but for some exceptions it destroys it again later when it pushes an error code. The x86-interrupt calling convention takes care of this by realigning the stack in this case.

- A guard page is a special memory page at the bottom of a stack that makes it possible to detect stack overflows. The page is not mapped to any physical frame, so accessing it causes a page fault instead of silently corrupting other memory. The bootloader sets up a guard page for our kernel stack, so a stack overflow causes a page fault.

