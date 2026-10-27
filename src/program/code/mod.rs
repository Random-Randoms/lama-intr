use std::{
    fmt::Display,
    ops::{Add, AddAssign},
};

use anyhow::{Context, anyhow};
use insn::Insn;
use log::debug;

use crate::{
    opnd::{Opnd, Value},
    program::{insn::LoadInsn, layout::Layout, load::Load, string_tab::StringTab},
};

pub mod insn;
pub mod loc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ip(usize);

impl Ip {
    pub fn into_opnd(self) -> Opnd {
        Value::Ptr(self.0).into_opnd()
    }
}

impl Display for Ip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[0x{:05x}]", self.0)
    }
}

impl Add<usize> for Ip {
    type Output = Self;
    fn add(self, other: usize) -> Self {
        Ip(self.0 + other)
    }
}

impl AddAssign<usize> for Ip {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

#[derive(Debug)]
pub struct Function {
    pub locals: u32,
    pub is_closure: bool,
}

struct InsnIter<'a> {
    code: &'a Code<'a>,
    off: usize,
}

impl<'a> Iterator for InsnIter<'a> {
    type Item = Insn;

    fn next(&mut self) -> Option<Self::Item> {
        self.code
            .get_insn_or_end(Ip(self.off))
            .ok()?
            .map(|(insn, ip)| {
                self.off = ip.0;
                insn
            })
    }
}

#[derive(Clone, Copy)]
pub struct Code<'a> {
    data: &'a [u8],
    string_tab: StringTab<'a>,
}

impl<'a> Code<'a> {
    pub fn new(data: &'a [u8], string_tab: StringTab<'a>) -> Self {
        Self { data, string_tab }
    }

    pub fn get_insn(&self, ip: Ip) -> anyhow::Result<(Insn, Ip)> {
        self.get_insn_or_end(ip)?
            .ok_or(anyhow!("Code area end reached at {:x}", ip.0))
    }

    fn get_insn_or_end(&self, ip: Ip) -> anyhow::Result<Option<(Insn, Ip)>> {
        Ok(self
            .data
            .load_insn(ip.0, self.string_tab)
            .context(format!("Couldn't load insn at {:x}", ip.0))?
            .map(|(insn, off)| (insn, Ip(off))))
    }

    pub fn get_insns(&self) -> impl Iterator<Item = Insn> {
        InsnIter { off: 0, code: self }
    }

    pub fn get_fun(&self, ip: Ip) -> anyhow::Result<Function> {
        match self
            .data
            .load_insn(ip.0, self.string_tab)
            .context("Couldn't get function definition for address")?
            .ok_or(anyhow!(
                "Code area end instead of function definition at {:x}",
                ip.0
            ))?
            .0
        {
            Insn::Begin {
                args: _,
                locals,
                is_closure,
            } => Ok(Function { locals, is_closure }),
            _ => Err(anyhow!("Function doesn't start ar this address")),
        }
    }
}

pub trait LoadIp: Load {
    fn load_ip(&self, off: usize) -> anyhow::Result<Ip>;
}

impl<T: Load + ?Sized> LoadIp for T {
    fn load_ip(&self, off: usize) -> anyhow::Result<Ip> {
        { Ok::<_, anyhow::Error>(Ip(self.load_i32(off)?.try_into()?)) }
            .context("Couldn't load instruction pointer")
    }
}

pub trait LoadCode: Load {
    fn load_code<'a>(
        &'a self,
        string_tab: StringTab<'a>,
        layout: &Layout,
    ) -> anyhow::Result<Code<'a>>;
}

impl<T: Load + ?Sized> LoadCode for T {
    fn load_code<'a>(
        &'a self,
        string_tab: StringTab<'a>,
        layout: &Layout,
    ) -> anyhow::Result<Code<'a>> {
        let off = layout.code_off;
        let mut to = off;
        loop {
            let Some((insn, next_to)) = self
                .load_insn(to, string_tab)
                .context(format!("Malformed insn in code area at {to:x}"))?
            else {
                break;
            };
            debug!("load insn {:25} at {to:#010x}", format!("{insn}"));
            to = next_to;
        }
        self.subslice(off, to - off)
            .context("Couldn't load code area")
            .map(|slice| Code::new(slice, string_tab))
    }
}
