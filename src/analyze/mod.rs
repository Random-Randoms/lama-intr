use std::{collections::HashMap, fmt::Display};

use crate::{
    analyze::{
        code_type::{CodeType, Type},
        seq::Seq,
    },
    program::{Code, insn::Insn},
};

mod code_type;
mod seq;

pub struct SeqData {
    data: Vec<(Seq, usize)>,
}

impl SeqData {
    pub fn from_code<'a>(code: Code<'a>) -> Self {
        let mut freq = HashMap::<Seq, usize>::new();
        code.useful_insns().for_each(|insn| {
            freq.add_one(Seq::One(insn));
        });
        code.useful_insns()
            .zip(code.useful_insns().skip(1))
            .for_each(|(fst, snd)| {
                if matches!(fst.get_type(), CodeType::Jump) {
                    return;
                }
                freq.add_one(Seq::Two(fst, snd));
            });
        let mut data = freq.into_iter().collect::<Vec<_>>();
        data.sort_by_key(|l| std::cmp::Reverse(l.1));
        Self { data }
    }
}

impl Display for SeqData {
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

trait AddOne<T> {
    fn add_one(&mut self, key: T);
}

impl AddOne<Seq> for HashMap<Seq, usize> {
    fn add_one(&mut self, key: Seq) {
        *self.entry(key).or_default() += 1;
    }
}

trait UsefulInsns {
    fn useful_insns(&self) -> impl Iterator<Item = Insn>;
}

impl<'a> UsefulInsns for Code<'a> {
    fn useful_insns(&self) -> impl Iterator<Item = Insn> {
        self.get_insns()
            .filter(|insn| !matches!(insn.get_type(), CodeType::Ignore))
    }
}
