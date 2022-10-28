
extern crate alloc;


use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::{Vec};

use log::info;
use uefi::data_types::CStr16;
use uefi::prelude::*;
use uefi::proto::media::file::{Directory, File, FileAttribute, FileMode, FileType, RegularFile};
use uefi::proto::media::fs::SimpleFileSystem;


pub struct CrawlEntry {
    pub path: String,
    pub file: RegularFile,
}

pub fn crawl_root(sfs_handle: Handle, system_table: &mut SystemTable<Boot>) -> Vec<Box<CrawlEntry>>{
    let fs = system_table
        .boot_services()
        .open_protocol_exclusive::<SimpleFileSystem>(
            sfs_handle
        )
        .expect("Could not handle protocol")
        .interface
        .get();

    let mut root = unsafe { (*fs).open_volume().expect("Could not open volume") };

    crawl_tree(system_table, &mut root, Vec::new())
}


pub fn crawl_tree(system_table: &mut SystemTable<Boot>, dir: &mut Directory, cur_path: Vec<String>) -> Vec<Box<CrawlEntry>>{

    let mut res = Vec::new();

    // "." and ".."
    let dir_self   = CStr16::from_u16_with_nul(&[0x002E, 0x0]).unwrap();
    let dir_parent = CStr16::from_u16_with_nul(&[0x002E, 0x002E, 0x0]).unwrap();

    let mut buf: Vec<u8> = Vec::new();

    loop {

        match dir.read_entry(&mut buf[..]).map_err(|err| err.split()) {

            Ok(ret) => {
                if let Some(f) = ret {
                    let filename = f.file_name();
                    // Compare with "." and ".."
                    if filename == dir_self 
                    || filename == dir_parent {
                        continue;
                    }

                    match dir
                        .handle()
                        .open(f.file_name(), FileMode::Read, FileAttribute::READ_ONLY)
                    {
                        Ok(f) => {
                            if let Ok(c) = f.into_type() {
                                let mut path = cur_path.to_owned();
                                path.push(filename.to_string());

                                match c {
                                    FileType::Dir(mut d) => {
                                        res.append(&mut crawl_tree(system_table, &mut d, path));
                                    }
                                    FileType::Regular(f) => {
                                        res.push(Box::new(
                                            CrawlEntry { path: path.join("/"), file: f }
                                        ))
                                    }
                                }
                            }
                        }
                        Err(_) => {}
                    }
                } else {
                    break;
                }
            }

            Err((_, Some(new_size))) => {
                buf.extend((0..new_size - buf.len()).map(|_| 0));
            }

            Err((status, None)) => panic!("Failed to read root dir. Status: {:?}", status),
        };
    }

    res
}

pub fn load_file(f: &mut RegularFile) -> Vec<u8> {
    let mut buf: Vec<u8> = alloc::vec![0; 512];
    let mut i: usize = 0;

    loop {

        match f.read(&mut buf[i..]).map_err(|err| err.split())  {

            Ok(ret) => {
                if ret == 0 {
                    break
                } else {
                    i += 512;
                    buf.extend(alloc::vec![0; 512]);
                }
            }

            Err((_, Some(new_size))) => {
                info!("Extend to: {} bytes", new_size);
                break
            }

            Err((status, None)) => panic!("Failed to read file. Status: {:?}", status),
        };
    }

    info!("Read file of {:?} bytes!", buf.len());
    buf
}

pub fn load_image_buf(handle: Handle, system_table: &mut SystemTable<Boot>, buf: Vec<u8>) {
    let img_handle = system_table.boot_services()
    .load_image(handle, uefi::table::boot::LoadImageSource::FromBuffer { buffer: &buf, file_path: None }).unwrap();

    system_table.boot_services()
    .start_image(img_handle).unwrap();
}