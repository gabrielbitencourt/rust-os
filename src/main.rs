#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ros::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use bootloader::BootInfo;
use x86_64::{
    VirtAddr,
    structures::paging::MapperAllSizes
};

extern crate alloc;
use alloc::boxed::Box;

use ros::memory::paging;
use ros::println;

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    #[cfg(test)]
    test_main();

    ros::init();
    let x = Box::new(41);
    println!("hello human");

    ros::halt();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    ros::halt();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ros::test_panic_handler(info);
}