use anyhow::Context;

use crate::program::{load::Load, public_tab};

use log::debug;

const INT_SIZE: usize = 0x4;
const STRING_TAB_LEN_LOC: usize = 0;
const GLOBAL_AREA_SIZE_LOC: usize = STRING_TAB_LEN_LOC + INT_SIZE;
const PUBLIC_SYMBOLS_NUM_LOC: usize = GLOBAL_AREA_SIZE_LOC + INT_SIZE;
const CODE_LOC: usize = PUBLIC_SYMBOLS_NUM_LOC + INT_SIZE;

#[derive(Debug, Clone, Copy)]
pub struct Segment {
    pub off: usize,
    pub len: usize,
}

impl Segment {
    fn end(&self) -> usize {
        self.off + self.len
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub public_tab: Segment,
    pub string_tab: Segment,
    pub global_area_elts: usize,
    pub code_off: usize,
}

impl Layout {
    fn new(string_tab_len: usize, global_area_elts: usize, public_symbols_num: usize) -> Self {
        let public_tab = Segment {
            off: CODE_LOC,
            len: public_symbols_num * public_tab::ENTRY_SIZE,
        };
        let string_tab = Segment {
            off: public_tab.end(),
            len: string_tab_len,
        };
        let code_off = string_tab.end();
        debug!(
            "Load layout:\n\
               public_tab: {:#010x}..{:#010x}\n\
               string_tab: {:#010x}..{:#010x}\n\
               code:       {:#010x}..",
            public_tab.off,
            public_tab.end(),
            string_tab.off,
            string_tab.end(),
            code_off
        );
        Self {
            public_tab,
            string_tab,
            global_area_elts,
            code_off,
        }
    }
}

pub trait LoadLayout: Load {
    fn load_layout(&self) -> anyhow::Result<Layout>;
}

impl<T: Load + ?Sized> LoadLayout for T {
    fn load_layout(&self) -> anyhow::Result<Layout> {
        let string_tab_size: usize = self
            .load_i32(STRING_TAB_LEN_LOC)
            .context("Couldn't load string table size")?
            .try_into()
            .context("String tab size is negative")?;
        let global_area_size: usize = self
            .load_i32(GLOBAL_AREA_SIZE_LOC)
            .context("Couldn't load global area size")?
            .try_into()
            .context("Global area size is negative")?;
        let public_symbols_num: usize = self
            .load_i32(PUBLIC_SYMBOLS_NUM_LOC)
            .context("Couldn't load public symbols number")?
            .try_into()
            .context("Public symbols number is negative")?;
        Ok(Layout::new(
            string_tab_size,
            global_area_size,
            public_symbols_num,
        ))
    }
}
