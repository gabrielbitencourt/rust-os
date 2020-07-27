#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]

use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use core::panic::PanicInfo;
use ros::{serial_print, serial_println, exit_qemu, QemuExitCode};

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow... ");

    ros::gdt::init();
    init_test_idt();
    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ros::test_panic_handler(info);
}

#[allow(unconditional_recursion)]
fn stack_overflow() -> u8 {
    let a = stack_overflow();
    return a;
}

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(ros::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        return idt;
    };
}

extern "x86-interrupt" fn test_double_fault_handler(stack_frame: &mut InterruptStackFrame, _error_code: u64) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

pub fn init_test_idt() {
    TEST_IDT.load();
}