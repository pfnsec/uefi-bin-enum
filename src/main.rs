#![no_main]
#![no_std]
#![feature(abi_efiapi)]

pub mod crawl;
pub mod menu;

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use uefi::prelude::*;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::SearchType;

use crawl::crawl_root;

use crate::crawl::CrawlEntry;
use crate::menu::MenuState;

#[entry]
fn main(handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    let boot_services = system_table.boot_services();

    let handles = boot_services
        .locate_handle_buffer(SearchType::from_proto::<SimpleFileSystem>())
        .expect("Could not find any SimpleFileSystem handle");

    let handles = handles.handles().clone();

    if handles.len() < 1 {
        panic!("Could not find any SimpleFileSystem handle");
    }

    let mut crawl_entries: Vec<Box<CrawlEntry>> = Vec::new();

    for sfs_handle in handles {
        unsafe {
            crawl_entries.append(&mut crawl_root(
                sfs_handle.clone(),
                &mut system_table.unsafe_clone(),
            ))
        }
    }

    let mut menu_state = MenuState::from_crawl_entries(crawl_entries);

    unsafe {
        menu_state.run(handle, system_table.unsafe_clone());
    }

    Status::SUCCESS
}
