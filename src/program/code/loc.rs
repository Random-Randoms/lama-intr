use std::fmt::Display;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Loc {
    Global(u32),
    Local(u32),
    Arg(u32),
    Captured(u32),
}

impl Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Global(ind) => write!(f, "G{ind:2}"),
            Self::Local(ind) => write!(f, "L{ind:2}"),
            Self::Arg(ind) => write!(f, "A{ind:2}"),
            Self::Captured(ind) => write!(f, "C{ind:2}"),
        }
    }
}
