use crate::program::insn::Insn;

pub enum CodeType {
    Ignore,
    Jump,
    NotJump,
}

pub trait Type {
    fn get_type(&self) -> CodeType;
}

impl Type for Insn {
    fn get_type(&self) -> CodeType {
        match self {
            Insn::Jmp(..) => CodeType::Jump,
            Insn::Cjmp(..) => CodeType::Jump,
            Insn::Begin { .. } => CodeType::Jump,
            Insn::End => CodeType::Jump,
            Insn::Ret => CodeType::Jump,
            Insn::Fail { .. } => CodeType::Jump,
            Insn::Line(..) => CodeType::Ignore,
            _ => CodeType::NotJump,
        }
    }
}
