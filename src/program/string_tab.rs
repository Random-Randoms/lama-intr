use std::{ffi::CStr, fmt::Display};

use anyhow::{Context, anyhow};

use crate::{
    program::{layout::Layout, load::Load},
    util,
};

#[derive(Clone, Copy)]
pub struct StringTab<'a> {
    data: &'a [u8],
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct StringPointer(usize);

impl Display for StringPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<0x{:05x}>", self.0)
    }
}

impl<'a> StringTab<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn get_string(&self, ptr: StringPointer) -> anyhow::Result<&'a str> {
        {
            CStr::from_bytes_until_nul(
                self.data
                    .get(ptr.0..)
                    .ok_or(anyhow!("Incorrect string pointer {ptr}"))?,
            )?
            .to_str()
        }
        .context(anyhow!("Incorrect string tab record"))
    }

    pub fn get_tag_hash(&self, ptr: StringPointer) -> anyhow::Result<usize> {
        util::hash(
            self.data
                .get(ptr.0..)
                .ok_or(anyhow!("String pointer out of range"))?
                .split(|c| *c == 0)
                .next()
                .expect("should be at least one split"),
        )
    }

    pub fn get_ptr(&self, ptr: StringPointer) -> anyhow::Result<usize> {
        Ok(self
            .data
            .get(ptr.0..)
            .ok_or(anyhow!("String index out of range"))?
            .as_ptr() as usize)
    }
}

pub trait LoadStringPointer: Load {
    fn load_string_pointer(&self, off: usize) -> anyhow::Result<StringPointer>;
}

impl<T: Load + ?Sized> LoadStringPointer for T {
    fn load_string_pointer(&self, off: usize) -> anyhow::Result<StringPointer> {
        self.load_i32(off)
            .context("Couldn't load string pointer")?
            .try_into()
            .context("String pointer negative")
            .map(StringPointer)
    }
}

pub trait LoadStringTab: Load {
    fn load_string_tab<'a>(&'a self, layout: &Layout) -> anyhow::Result<StringTab<'a>>;
}

impl<T: Load + ?Sized> LoadStringTab for T {
    fn load_string_tab<'a>(&'a self, layout: &Layout) -> anyhow::Result<StringTab<'a>> {
        self.subslice(layout.string_tab.off, layout.string_tab.len)
            .context("String tab overflows file")
            .map(StringTab::new)
    }
}
