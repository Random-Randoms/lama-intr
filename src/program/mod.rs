use crate::program::{
    code::LoadCode,
    layout::LoadLayout,
    public_tab::{LoadPublicTab, PublicTab},
    string_tab::{LoadStringTab, StringTab},
};

mod code;
mod layout;
mod load;
mod public_tab;
mod string_tab;

pub use code::{Code, Ip, insn, loc};

pub struct Program<'a> {
    pub string_tab: StringTab<'a>,
    pub global_area_elts: usize,
    pub code: Code<'a>,
    pub public_tab: PublicTab<'a>,
    pub entry_point: public_tab::Entry,
}

impl<'a> Program<'a> {
    pub fn load(data: &'a [u8]) -> anyhow::Result<Self> {
        let layout = data.load_layout()?;
        let string_tab: StringTab<'a> = data.load_string_tab(&layout)?;
        let code = data.load_code(string_tab, &layout)?;
        let public_tab = data.load_public_tab(&layout)?;
        let entry_point = public_tab.get_entry_point(string_tab)?;
        Ok(Self {
            string_tab,
            global_area_elts: layout.global_area_elts,
            code,
            public_tab,
            entry_point,
        })
    }
}
