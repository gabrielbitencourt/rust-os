use core::fmt::{Arguments,Result,Write};

use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        color_code: ColorCode::new(Color::Black, Color::Green),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
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

pub fn _print(args: Arguments) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(background: Color, foreground: Color) -> ColorCode {
        return ColorCode((background as u8) << 4 | foreground as u8);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

pub struct Writer {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {

    fn new_line(&mut self) {
        self.buffer.chars[self.row_position][self.column_position] = ScreenChar {
            ascii_character: b'\n',
            color_code: ColorCode::new(Color::Black, Color::Black)
        };
        self.row_position = self.row_position + 1;
        self.column_position = 0;
        if self.row_position == BUFFER_HEIGHT {
            self.row_position = BUFFER_HEIGHT - 1;
            for row in 0..BUFFER_HEIGHT - 1 {
                self.buffer.chars[row] = self.buffer.chars[row + 1];
            }
            self.clear_row(BUFFER_HEIGHT - 1);
        }
        self.light_up(self.row_position, self.column_position);
    }

    pub fn move_up(&mut self) {
        if self.row_position > 0 {
            self.light_down(self.row_position, self.column_position);
            self.row_position = self.row_position - 1;
            self.light_up(self.row_position, self.column_position);
        }
    }

    pub fn move_down(&mut self) {
        if self.row_position < BUFFER_HEIGHT - 1 {
            self.light_down(self.row_position, self.column_position);
            self.row_position = self.row_position + 1;
            self.light_up(self.row_position, self.column_position);
        }
    }

    pub fn move_right(&mut self) {
        self.light_down(self.row_position, self.column_position);
        if self.column_position < BUFFER_WIDTH - 1 {
            self.column_position = self.column_position + 1;
        }
        else {
            self.row_position = self.row_position + 1;
            self.column_position = 0;
        }
        self.light_up(self.row_position, self.column_position);
    }

    pub fn move_left(&mut self) {
        self.light_down(self.row_position, self.column_position);
        if self.column_position > 0 {
            self.column_position = self.column_position - 1;
        }
        else {
            self.row_position = self.row_position - 1;
            self.column_position = BUFFER_WIDTH - 1;
        }
        self.light_up(self.row_position, self.column_position);
    }

    fn clear_row(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col] = ScreenChar {
                ascii_character: 0,
                color_code: ColorCode::new(Color::Black, Color::Black)
            };
        }
    }

    pub fn light_down(&mut self, row: usize, col: usize) {
        self.buffer.chars[row][col] = ScreenChar {
            ascii_character: self.buffer.chars[row][col].ascii_character,
            color_code: ColorCode::new(Color::Black, Color::Green)
        };
    }

    pub fn light_up(&mut self, row: usize, col: usize) {
        self.buffer.chars[row][col] = ScreenChar {
            ascii_character: self.buffer.chars[row][col].ascii_character,
            color_code: ColorCode::new(Color::Green, Color::Black)
        };
    }

    pub fn backspace(&mut self) {
        self.light_down(self.row_position, self.column_position);
        if self.column_position == 0 {
            self.row_position = self.row_position - 1;
            for col in (0..BUFFER_WIDTH).rev() {
                if self.buffer.chars[self.row_position][col].ascii_character == b'\n' {
                    self.column_position = col + 1;
                    break;
                }
            }
            self.backspace();
        }
        else {
            self.buffer.chars[self.row_position][self.column_position - 1] = ScreenChar {
                ascii_character: 0,
                color_code: ColorCode::new(Color::Black, Color::Black)
            };
            self.column_position = self.column_position - 1;
        }
        self.light_up(self.row_position, self.column_position);
    }

    pub fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            match byte {
                0x20..=0x7e => self.write(byte),
                b'\n' => self.new_line(),
                _ => self.write(0xfe)
            }
        }
    }

    pub fn write(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position == BUFFER_WIDTH - 1 {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code
                };
                self.column_position = self.column_position + 1;
                self.light_up(self.row_position, self.column_position);
            }
        }
    }
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> Result {
        self.write_string(s);
        return Ok(());
    }
}

#[test_case]
fn print_simple() {
    print!("simple print: {}", 123);
}

#[test_case]
fn print_multiple() {
    print!("\n");
    for i in 0..=3 {
        print!("multiple print: {}, ", i);
    }
}

#[test_case]
fn println_simple() {
    println!("simple println: {}", 123);
}

#[test_case]
fn println_multiple() {
    for i in 0..200 {
        println!("multiple println: {}, ", i);
    }
}

#[test_case]
fn test_println_output() {
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s);
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i];
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}