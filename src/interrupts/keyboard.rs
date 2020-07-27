use crate::{print, serial_println};
use crate::vga_buffer::WRITER;
use super::{eoi, InterruptIndex,Mutex};

use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptStackFrame;

struct Keyboard {
    capslock: bool,
    shifted: bool
}

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard::new());
}

impl Keyboard {

    fn new() -> Keyboard {
        return Keyboard {
            capslock: false,
            shifted: false
        }
    }

    fn parse(&mut self, code: u8) -> Option<char> {
        return match (code, self.capslock, self.shifted) {
            (0x02..=0x0d, _, false) => match code {
                0x02 => Some('1'),
                0x03 => Some('2'),
                0x04 => Some('3'),
                0x05 => Some('4'),
                0x06 => Some('5'),
                0x07 => Some('6'),
                0x08 => Some('7'),
                0x09 => Some('8'),
                0x0a => Some('9'),
                0x0b => Some('0'),
                0x0C => Some('-'),
                0x0D => Some('='),
                _ => unreachable!()
            },
            (0x02..=0x0d, _, true) => match code {
                0x02 => Some('!'),
                0x03 => Some('@'),
                0x04 => Some('#'),
                0x05 => Some('$'),
                0x06 => Some('%'),
                0x07 => Some('^'),
                0x08 => Some('&'),
                0x09 => Some('*'),
                0x0a => Some('('),
                0x0b => Some(')'),
                0x0c => Some('_'),
                0x0d => Some('+'),
                _ => unreachable!()
            },
            (0x0E, _, _) => {
                x86_64::instructions::interrupts::without_interrupts(|| {
                    WRITER.lock().backspace();
                });
                return None;
            },
            (0x0F, _, _) => Some('\t'), // tab
            (0x1C, _, _) => Some('\n'), // enter
            (0x10..=0x19 | 0x1e..=0x26 | 0x2c..=0x32, false, false) => match code {
                0x10 => Some('q'),
                0x11 => Some('w'),
                0x12 => Some('e'),
                0x13 => Some('r'),
                0x14 => Some('t'),
                0x15 => Some('y'),
                0x16 => Some('u'),
                0x17 => Some('i'),
                0x18 => Some('o'),
                0x19 => Some('p'),

                0x1E => Some('a'),
                0x1F => Some('s'),
                0x20 => Some('d'),
                0x21 => Some('f'),
                0x22 => Some('g'),
                0x23 => Some('h'),
                0x24 => Some('j'),
                0x25 => Some('k'),
                0x26 => Some('l'),
                
                0x2C => Some('z'),
                0x2D => Some('x'),
                0x2E => Some('c'),
                0x2F => Some('v'),
                0x30 => Some('b'),
                0x31 => Some('n'),
                0x32 => Some('m'),
                _ => unreachable!()
            },
            (0x10..=0x19 | 0x1e..=0x26 | 0x2c..=0x32, true, _) | (0x10..=0x19 | 0x1e..=0x26 | 0x2c..=0x32, _, true) => match code {
                0x10 => Some('Q'),
                0x11 => Some('W'),
                0x12 => Some('E'),
                0x13 => Some('R'),
                0x14 => Some('T'),
                0x15 => Some('Y'),
                0x16 => Some('U'),
                0x17 => Some('I'),
                0x18 => Some('O'),
                0x19 => Some('P'),

                0x1E => Some('A'),
                0x1F => Some('S'),
                0x20 => Some('D'),
                0x21 => Some('F'),
                0x22 => Some('G'),
                0x23 => Some('H'),
                0x24 => Some('J'),
                0x25 => Some('K'),
                0x26 => Some('L'),

                0x2C => Some('Z'),
                0x2D => Some('X'),
                0x2E => Some('C'),
                0x2F => Some('V'),
                0x30 => Some('B'),
                0x31 => Some('N'),
                0x32 => Some('M'),
                _ => unreachable!()
            },
            (0x1A, _, false) => Some('['),
            (0x1A, _, true) => Some('{'),
            (0x1B, _, false) => Some(']'),
            (0x1B, _, true) => Some('}'),
            (0x33, _, false) => Some(','),
            (0x33, _, true) => Some('<'),
            (0x34, _, false) => Some('.'),
            (0x34, _, true) => Some('>'),
            (0x27, _, false) => Some(';'),
            (0x27, _, true) => Some(':'),
            (0x28, _, false) => Some('\''),
            (0x28, _, true) => Some('"'),
            (0x29, _, false) => Some('`'),
            (0x29, _, true) => Some('~'),
            (0x2B, _, false) => Some('\\'),
            (0x2B, _, true) => Some('|'),
            (0x35, _, false) => Some('/'),
            (0x35, _, true) => Some('?'),
            (0x39, _, _) => Some(' '),
            (0x2A, _, false) => { // shift right
                self.shifted = true;
                return None
            },
            (0x36, _, false) => { // shift left
                self.shifted = true;
                return None
            },
            (0xb6, _, true) => { // unshift
                self.shifted = false;
                return None;
            },
            (0xba, _, _) => { // capslock (binded to key release)
                self.capslock = !self.capslock;
                return None;
            },
            (0x48, _, _) => { // arrow up
                x86_64::instructions::interrupts::without_interrupts(|| {
                    serial_println!("arrow up");
                    WRITER.lock().move_up();
                });
                return None;
            },
            (0x4b, _, _) => { // arrow left
                x86_64::instructions::interrupts::without_interrupts(|| {
                    serial_println!("arrow left");
                    WRITER.lock().move_left();
                });
                return None;
            },
            (0x4d, _, _) => { // arrow right
                x86_64::instructions::interrupts::without_interrupts(|| {
                    serial_println!("arrow right");
                    WRITER.lock().move_right();
                });
                return None;
            },
            (0x50, _, _) => { // arrow down
                x86_64::instructions::interrupts::without_interrupts(|| {
                    serial_println!("arrow down");
                    WRITER.lock().move_down();
                });
                return None;
            },
            (0x81..=0xff, _, _) => None,
            (_, _, _) => None
        }
    }
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    if let Some(key) = KEYBOARD.lock().parse(scancode) {
        print!("{}", key);
    }
    unsafe {
        eoi(InterruptIndex::Keyboard as u8);
    }
}