
extern crate alloc;


use alloc::boxed::Box;
use alloc::vec::Vec;
use log::info;
use core::cmp;
use core::fmt::Write;
use uefi::{prelude::*, Char16};
use uefi::proto::console::text::{Color, ScanCode};
use uefi::proto::console::text::Key::*;

use crate::crawl::{CrawlEntry, self};

pub struct MenuState {
    pub entries: Vec<Box<CrawlEntry>>,
    pub current_item: usize
}

impl MenuState {
    pub fn from_crawl_entries(entries: Vec<Box<CrawlEntry>>) -> Self {
        let state = MenuState {
            entries: entries,
            current_item: 0
        };

        state
    }

    pub fn render(&self, mut system_table: SystemTable<Boot>) {
        let stdout = system_table.stdout();

        stdout
            .set_color(Color::White, Color::Black)
            .expect("Failed to change console color");
        stdout.clear().expect("Failed to clear screen");
        for (i, entry) in self.entries.iter().enumerate() {
            stdout
                .set_color(Color::Green, Color::Black).unwrap();

            if i == self.current_item {
                stdout
                    .set_color(Color::Green, Color::Brown).unwrap();
            }
            stdout.write_str(&**&entry.path).unwrap();
            stdout.write_str("\n").unwrap();

            stdout
                .set_color(Color::Green, Color::Black).unwrap();
        }
    }

    pub fn run(&mut self, handle: Handle, mut system_table: SystemTable<Boot>) {
        let newline: Char16 = Char16::try_from('\r').unwrap();
        unsafe {
            self.render(system_table.unsafe_clone());
        }

        loop {
            let _ev = system_table.stdin().wait_for_key_event();
            let key = system_table.stdin().read_key().unwrap();
            match key {
                Some(k) => {
                    info!("{:?}", k);
                    match k {
                        Printable(char) => {
                            if char == newline {
                                info!("Executing image at {}", self.entries[self.current_item].path);
                                let buf = crawl::load_file(&mut self.entries[self.current_item].file);
                                crawl::load_image_buf(handle, &mut system_table, buf);
                            }
                            
                        }
                        Special(ScanCode::UP) => {
                            if self.current_item > 0 {
                                self.current_item -= 1;
                            }

                            unsafe {
                                self.render(system_table.unsafe_clone());
                            }
                        }
                        Special(ScanCode::DOWN) => {
                            self.current_item = cmp::min(self.current_item + 1, self.entries.len() - 1);

                            unsafe {
                                self.render(system_table.unsafe_clone());
                            }
                        }
                        Special(ScanCode::ESCAPE) => {
                            break;
                        }
                        _ => {}
                    }
                }
                None => {}
            }
        }
    }
}