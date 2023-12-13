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

- Segmentation was already introduced in 1978, originally to increase the amount of addressable memory. The situation back then was that CPUs only used 16-bit addresses, which limited the amount of addressable memory to 64 KiB. To make more than these 64 KiB accessible, additional segment registers were introduced, each containing an offset address. The CPU automatically added this offset on each memory access, so that up to 1 MiB of memory was accessible.

![Virtual Address Translation](/assets/virt-to-phys-addr.png)

- Another advantage is that programs can now be placed at arbitrary physical memory locations, even if they use completely different virtual addresses. Thus, the OS can utilize the full amount of available memory without needing to recompile programs.

- One way to combat this fragmentation is to pause execution, move the used parts of the memory closer together, update the translation, and then resume execution:

![Defragmentation of Sorts](/assets/defrag.png)

- The disadvantage of this defragmentation process is that it needs to copy large amounts of memory, which decreases performance. It also needs to be done regularly before the memory becomes too fragmented. This makes performance unpredictable since programs are paused at random times and might become unresponsive.

- The fragmentation problem is one of the reasons that segmentation is no longer used by most systems. In fact, segmentation is not even supported in 64-bit mode on x86 anymore. Instead, paging is used, which completely avoids the fragmentation problem.

- The idea is to divide both the virtual and physical memory space into small, fixed-size blocks. The blocks of the virtual memory space are called pages, and the blocks of the physical address space are called frames. Each page can be individually mapped to a frame, which makes it possible to split larger memory regions across non-continuous physical frames. The advantage of this becomes visible if we recap the example of the fragmented memory space, but use paging instead of segmentation this time:

![Paging](/assets/paging.png)

- Compared to segmentation, paging uses lots of small, fixed-sized memory regions instead of a few large, variable-sized regions. Since every frame has the same size, there are no frames that are too small to be used, so no fragmentation occurs... Or it seems like no fragmentation occurs. There is still some hidden kind of fragmentation, the so-called internal fragmentation. Internal fragmentation occurs because not every memory region is an exact multiple of the page size. Imagine a program of size 101 in the above example: It would still need three pages of size 50, so it would occupy 49 bytes more than needed. To differentiate the two types of fragmentation, the kind of fragmentation that happens when using segmentation is called external fragmentation.

- Internal fragmentation is unfortunate but often better than the external fragmentation that occurs with segmentation. It still wastes memory, but does not require defragmentation and makes the amount of fragmentation predictable (on average half a page per memory region).

- We saw that each of the potentially millions of pages is individually mapped to a frame. This mapping information needs to be stored somewhere. Segmentation uses an individual segment selector register for each active memory region, which is not possible for paging since there are way more pages than registers. Instead, paging uses a table structure called page table to store the mapping information.

![Page Tables](/assets/page-tables.png)

- To reduce the wasted memory, we can use a two-level page table. The idea is that we use different page tables for different address regions. An additional table called level 2 page table contains the mapping between address regions and (level 1) page tables.

- This is best explained by an example. Let’s define that each level 1 page table is responsible for a region of size 10_000. Then the following tables would exist for the above example mapping:

![Two Level Page Table](/assets/two-level-page-table.png)

- The principle of two-level page tables can be extended to three, four, or more levels. Then the page table register points to the highest level table, which points to the next lower level table, which points to the next lower level, and so on. The level 1 page table then points to the mapped frame. The principle in general is called a multilevel or hierarchical page table.

- The x86_64 architecture uses a 4-level page table and a page size of 4 KiB. Each page table, independent of the level, has a fixed size of 512 entries. Each entry has a size of 8 bytes, so each table is 512 * 8 B = 4 KiB large and thus fits exactly into one page.

- The page table index for each level is derived directly from the virtual address:

![Page Table Index](/assets/page-table-index.png)

- We see that each table index consists of 9 bits, which makes sense because each table has 2^9 = 512 entries. The lowest 12 bits are the offset in the 4 KiB page (2^12 bytes = 4 KiB). Bits 48 to 64 are discarded, which means that x86_64 is not really 64-bit since it only supports 48-bit addresses.

![Example Translation](/assets/example-translation.png)

- The above page table hierarchy maps two pages (in blue). From the page table indices, we can deduce that the virtual addresses of these two pages are 0x803FE7F000 and 0x803FE00000. Let’s see what happens when the program tries to read from address 0x803FE7F5CE. First, we convert the address to binary and determine the page table indices and the page offset for the address:

![Virtual Address](/assets/virtual-address.png)

- We start by reading the address of the level 4 table out of the CR3 register.
- The level 4 index is 1, so we look at the entry with index 1 of that table, which tells us that the level 3 table is stored at address 16 KiB.
- We load the level 3 table from that address and look at the entry with index 0, which points us to the level 2 table at 24 KiB.
- The level 2 index is 511, so we look at the last entry of that page to find out the address of the level 1 table.
- Through the entry with index 127 of the level 1 table, we finally find out that the page is mapped to frame 12 KiB, or 0x3000 in hexadecimal.
- The final step is to add the page offset to the frame address to get the physical address 0x3000 + 0x5ce = 0x35ce.

![Example Translation More Detail](/assets/example-translation-more-detail.png)

- It’s important to note that even though this example used only a single instance of each table, there are typically multiple instances of each level in each address space. At maximum, there are:

- one level 4 table,
- 512 level 3 tables (because the level 4 table has 512 entries),
- 512 * 512 level 2 tables (because each of the 512 level 3 tables has 512 entries), and
- 512 * 512 * 512 level 1 tables (512 entries for each level 2 table).

- Each page table entry is 8 bytes (64 bits) large and has the following format:
```
Bit(s)	    Name	                Meaning
0           present	                the page is currently in memory
1	        writable	            it’s allowed to write to this page
2	        user accessible	        if not set, only kernel mode code can access this page
3	        write-through caching	writes go directly to memory
4	        disable cache	        no cache is used for this page
5	        accessed	            the CPU sets this bit when this page is used
6	        dirty	                the CPU sets this bit when a write to this page occurs
7	        huge page/null	        must be 0 in P1 and P4, creates a 1 GiB page in P3, creates a 2 MiB page in P2
8	        global	                page isn’t flushed from caches on address space switch (PGE bit of CR4 register must be set)
9-11	    available	            can be used freely by the OS
12-51	    physical address	    the page aligned 52bit physical address of the frame or the next page table
52-62	    available	            can be used freely by the OS
63	        no execute	            forbid executing code on this page (the NXE bit in the EFER register must be set)
```

- A 4-level page table makes the translation of virtual addresses expensive because each translation requires four memory accesses. To improve performance, the x86_64 architecture caches the last few translations in the so-called translation lookaside buffer (TLB). This allows skipping the translation when it is still cached.

- One thing that we did not mention yet: Our kernel already runs on paging. The bootloader that we added in the “A minimal Rust Kernel” post has already set up a 4-level paging hierarchy that maps every page of our kernel to a physical frame. The bootloader does this because paging is mandatory in 64-bit mode on x86_64. This means that every memory address that we used in our kernel was a virtual address. Accessing the VGA buffer at address 0xb8000 only worked because the bootloader identity mapped that memory page, which means that it mapped the virtual page 0xb8000 to the physical frame 0xb8000.

