#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![deny(unused_must_use)]

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate log;

use uefi::prelude::*;
// use uefi::table::boot::*;

#[entry]
fn efi_main(_image: uefi::Handle, mut st: SystemTable<Boot>) -> Status {
    // Initialize utilities (logging, memory allocation...)
    uefi_services::init(&mut st).expect_success("failed to initialize utilities");
    let bs = st.boot_services();

    // log::set_max_level(log::LevelFilter::Debug);
    panic!("end");
    // Status::SUCCESS
}
