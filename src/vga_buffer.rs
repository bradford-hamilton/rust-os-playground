use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// We use a C-like enum here to explicitly specify the number for each color.
// Because of the repr(u8) attribute, each enum variant is stored as a u8.
// Actually 4 bits would be sufficient, but Rust doesn’t have a u4 type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

// To ensure that the ColorCode has the exact same data layout
// as a u8, we use the repr(transparent) attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_char: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// repr[transparent] here to ensure that it has the same
// memory layout as its single field.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    // Write a single byte to the VGA buffer/display.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_char: byte,
                    color_code: color_code,
                });

                self.column_position += 1;
            }
        }
    }

    // Write a string to the VGA buffer/display.
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Printable ASCII byte or newline:
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Not part of printable ASCII range.
                _ => self.write_byte(0xfe),
            }
        }
    }

    // Creates a newline on the display, shifting everthing up by one.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let char = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(char);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    // Clears a row by overwriting all of its characters with a space character.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_char: b' ',
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

// Implement fmt::Write for Writer so we can use Rust’s built-in write!/writeln! formatting macros.
impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// The one-time initialization of statics with non-const functions is a common
// problem in Rust. Fortunately, there already exists a good solution in a crate
// named lazy_static. This crate provides a lazy_static! macro that defines a
// lazily initialized static. Instead of computing its value at compile time,
// the static lazily initializes itself when accessed for the first time. Thus,
// the initialization happens at runtime, so arbitrarily complex initialization
// code is possible. we can use the spinning mutex to add safe interior mutability
// to our static WRITER:
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xB8000 as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// Since the macros need to be able to call _print from outside of the module,
// the function needs to be public. However, since we consider this a private
// implementation detail, we add the doc(hidden) attribute to hide it from the
// generated documentation.
#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[test_case]
fn test_println_no_panic() {
    println!("...printing!");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("so much printing");
    }
}

// Since println prints to the last screen line and then immediately appends
// a newline, the string should appear on line BUFFER_HEIGHT - 2.
#[test_case]
fn test_println_output() {
    let s = "some test string that fits on a single line";
    println!("{}", s);

    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_char), c);
    }
}
