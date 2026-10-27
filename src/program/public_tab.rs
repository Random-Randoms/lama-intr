use anyhow::Context;
use log::debug;

use crate::program::{
    Ip,
    code::LoadIp,
    layout::Layout,
    load::Load,
    string_tab::{LoadStringPointer, StringPointer, StringTab},
};

pub const ENTRY_SIZE: usize = 9;
const ENTRY_POINT_NAME: &str = "main";

pub struct Entry {
    pub name_off: StringPointer,
    pub code_off: Ip,
    pub is_function: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct PublicPointer(usize);

pub struct PublicTab<'a> {
    data: &'a [u8],
}

impl<'a> PublicTab<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn get(&self, ptr: PublicPointer) -> anyhow::Result<Entry> {
        self.data.load_entry(ptr.0 * ENTRY_SIZE)
    }

    pub fn get_entry_point<'b>(&self, string_tab: StringTab<'b>) -> anyhow::Result<Entry> {
        let mut cur = 0usize;
        loop {
            match self.data.load_entry(cur) {
                Ok(entry) => {
                    let name = string_tab.get_string(entry.name_off)?;
                    debug!("Entry: {name} at {}", entry.code_off);
                    if name == ENTRY_POINT_NAME {
                        break Ok(entry);
                    } else {
                        cur += ENTRY_SIZE;
                        continue;
                    }
                }
                Err(e) => Err(e),
            }
            .context("Couldn't find entry point")?
        }
    }
}

trait LoadEntry: Load {
    fn load_entry(&self, off: usize) -> anyhow::Result<Entry>;
}

impl<T: Load + ?Sized> LoadEntry for T {
    fn load_entry(&self, off: usize) -> anyhow::Result<Entry> {
        {
            Ok::<_, anyhow::Error>(Entry {
                name_off: self.load_string_pointer(off)?,
                code_off: self.load_ip(off + 4)?,
                is_function: self.load_u8(off + 8)? == 0,
            })
        }
        .context("Couldn't load public entry at {off}")
    }
}

pub trait LoadPublicTab: Load {
    fn load_public_tab<'a>(&'a self, layout: &Layout) -> anyhow::Result<PublicTab<'a>>;
}

impl<T: Load + ?Sized> LoadPublicTab for T {
    fn load_public_tab<'a>(&'a self, layout: &Layout) -> anyhow::Result<PublicTab<'a>> {
        {
            Ok::<_, anyhow::Error>(PublicTab::new(
                self.subslice(layout.public_tab.off, layout.public_tab.len)?,
            ))
        }
        .context("Couldn't load public tab at")
    }
}
