use std::fmt::Display;

use crate::program::insn::Insn;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Seq {
    One(Insn),
    Two(Insn, Insn),
}

impl Display for Seq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Seq::One(insn) => {
                let st = format!("{insn}");
                write!(f, "{st:25} | {:25}", "")
            }
            Seq::Two(fst, snd) => {
                let st1 = format!("{fst}");
                let st2 = format!("{snd}");
                write!(f, "{st1:25} | {st2:25}")
            }
        }
    }
}
