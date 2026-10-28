use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use crate::analyze::{
    insn::{Code, Instruction},
    seq::Seq,
};

pub mod insn;
mod seq;

pub struct SeqData<T: Instruction> {
    data: Vec<(Seq<T>, usize)>,
}

impl<'a, T: Instruction> SeqData<T> {
    pub fn from_code<U: Code<'a, T>>(code: &'a U) -> Self {
        let mut freqs = HashMap::<Seq<T>, usize>::new();
        let entries = code
            .instructions()
            .flat_map(|insn| {
                *freqs.entry(Seq::One(insn.clone())).or_default() += 1;
                insn.jumps().collect::<Vec<_>>()
            })
            .chain(code.entry_points())
            .collect::<HashSet<_>>();
        code.instructions()
            .zip(code.instructions().skip(1))
            .filter(|(fst, snd)| !fst.is_return() && !entries.contains(&snd.addr()))
            .for_each(|(fst, snd)| *freqs.entry(Seq::Two(fst, snd)).or_default() += 1);
        let mut data = freqs.into_iter().collect::<Vec<_>>();
        data.sort_by_key(|l| std::cmp::Reverse(l.1));
        Self { data }
    }
}

impl<T: Instruction> Display for SeqData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{:25} | {:25}| #occurencies",
            "first bytecode", "second bytecode"
        )?;
        writeln!(
            f,
            "--------------------------+--------------------------+-------------"
        )?;
        self.data
            .iter()
            .map(|(seq, cnt)| writeln!(f, "{seq}| {cnt:5}"))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }
}
