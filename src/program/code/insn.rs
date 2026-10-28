use std::{fmt::Display, hash::Hash, iter};

use anyhow::{Context, anyhow, bail};
use log::debug;

use crate::{
    opnd::{Opnd, Value},
    program::{
        Ip,
        code::{LoadIp, loc::Loc},
        load::Load,
        string_tab::{LoadStringPointer, StringPointer, StringTab},
    },
    runtime,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ArithBinop {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Lt,
    Le,
    Gt,
    Ge,
    Ne,
    And,
    Or,
}

impl Display for ArithBinop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Mod => write!(f, "%"),
            Self::Lt => write!(f, "<"),
            Self::Le => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::Ge => write!(f, ">="),
            Self::Ne => write!(f, "!="),
            Self::And => write!(f, "&&"),
            Self::Or => write!(f, "||"),
        }
    }
}

impl ArithBinop {
    fn eval(&self, lhs: isize, rhs: isize) -> isize {
        match self {
            ArithBinop::Add => lhs + rhs,
            ArithBinop::Sub => lhs - rhs,
            ArithBinop::Mul => lhs * rhs,
            ArithBinop::Div => lhs / rhs,
            ArithBinop::Mod => lhs % rhs,
            ArithBinop::Lt => (lhs < rhs).to_isize(),
            ArithBinop::Le => (lhs <= rhs).to_isize(),
            ArithBinop::Gt => (lhs > rhs).to_isize(),
            ArithBinop::Ge => (lhs >= rhs).to_isize(),
            ArithBinop::Ne => (lhs != rhs).to_isize(),
            ArithBinop::And => (lhs.to_bool() && rhs.to_bool()).to_isize(),
            ArithBinop::Or => (lhs.to_bool() || rhs.to_bool()).to_isize(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Binop {
    Eq,
    Arith(ArithBinop),
}

impl Display for Binop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eq => write!(f, "=="),
            Self::Arith(b) => write!(f, "{b}"),
        }
    }
}

trait ToBool {
    fn to_bool(self) -> bool;
}

impl ToBool for isize {
    fn to_bool(self) -> bool {
        self != 0
    }
}

trait ToIsize {
    fn to_isize(self) -> isize;
}

impl ToIsize for bool {
    fn to_isize(self) -> isize {
        if self { 1 } else { 0 }
    }
}

const ADD_LOW: u8 = 1;
const SUB_LOW: u8 = 2;
const MUL_LOW: u8 = 3;
const DIV_LOW: u8 = 4;
const MOD_LOW: u8 = 5;
const LT_LOW: u8 = 6;
const LE_LOW: u8 = 7;
const GT_LOW: u8 = 8;
const GE_LOW: u8 = 9;
const EQ_LOW: u8 = 10;
const NE_LOW: u8 = 11;
const AND_LOW: u8 = 12;
const OR_LOW: u8 = 13;

impl Binop {
    pub fn eval(&self, lhs: Opnd, rhs: Opnd) -> anyhow::Result<Opnd> {
        match self {
            Binop::Eq => Ok(Value::Int((lhs.raw() == rhs.raw()).to_isize()).into_opnd()),
            Binop::Arith(op) => {
                let lhs = lhs.into_value().into_int()?;
                let rhs = rhs.into_value().into_int()?;
                let result = op.eval(lhs, rhs);
                debug!("Binop: {lhs} , {rhs} -> {result}");
                Ok(Value::Int(result).into_opnd())
            }
        }
    }

    fn from_low(low: u8) -> anyhow::Result<Self> {
        match low {
            ADD_LOW => Ok(Binop::Arith(ArithBinop::Add)),
            SUB_LOW => Ok(Binop::Arith(ArithBinop::Sub)),
            MUL_LOW => Ok(Binop::Arith(ArithBinop::Mul)),
            DIV_LOW => Ok(Binop::Arith(ArithBinop::Div)),
            MOD_LOW => Ok(Binop::Arith(ArithBinop::Mod)),
            LT_LOW => Ok(Binop::Arith(ArithBinop::Lt)),
            LE_LOW => Ok(Binop::Arith(ArithBinop::Le)),
            GT_LOW => Ok(Binop::Arith(ArithBinop::Gt)),
            GE_LOW => Ok(Binop::Arith(ArithBinop::Ge)),
            EQ_LOW => Ok(Binop::Eq),
            NE_LOW => Ok(Binop::Arith(ArithBinop::Ne)),
            AND_LOW => Ok(Binop::Arith(ArithBinop::And)),
            OR_LOW => Ok(Binop::Arith(ArithBinop::Or)),
            _ => Err(anyhow!("Incorrect low opcode for binop")),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum JumpCondition {
    Zero,
    NotZero,
}

impl JumpCondition {
    pub fn eval(self, val: Opnd) -> anyhow::Result<bool> {
        match self {
            JumpCondition::Zero => Ok(val.into_value().into_int()? == 0),
            JumpCondition::NotZero => Ok(val.into_value().into_int()? != 0),
        }
    }
}

impl Display for JumpCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zero => write!(f, "z"),
            Self::NotZero => write!(f, "nz"),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Pattern {
    Unary(UnaryPattern),
    Binary(BinaryPattern),
}

impl Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unary(pat) => write!(f, "{pat}"),
            Self::Binary(pat) => write!(f, "{pat}"),
        }
    }
}

const STR_CMP_PAT_LOW: u8 = 0;
const STRING_PAT_LOW: u8 = 1;
const ARRAY_PAT_LOW: u8 = 2;
const SEXP_PAT_LOW: u8 = 3;
const BOXED_PAT_LOW: u8 = 4;
const UNBOXED_PAT_LOW: u8 = 5;
const CLOSURE_PAT_LOW: u8 = 6;

impl Pattern {
    fn from_low(low: u8) -> anyhow::Result<Self> {
        Ok(match low {
            STR_CMP_PAT_LOW => Pattern::Binary(BinaryPattern::StrCmp),
            STRING_PAT_LOW => Pattern::Unary(UnaryPattern::String),
            ARRAY_PAT_LOW => Pattern::Unary(UnaryPattern::Array),
            SEXP_PAT_LOW => Pattern::Unary(UnaryPattern::Sexp),
            BOXED_PAT_LOW => Pattern::Unary(UnaryPattern::Boxed),
            UNBOXED_PAT_LOW => Pattern::Unary(UnaryPattern::Unboxed),
            CLOSURE_PAT_LOW => Pattern::Unary(UnaryPattern::Closure),
            _ => bail!("Incorrect low opcode {low:x} for pattern"),
        })
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum BinaryPattern {
    StrCmp,
}

impl BinaryPattern {
    pub fn eval(&self, lhs: Opnd, rhs: Opnd) -> Opnd {
        runtime::builtin::string_patt(lhs, rhs)
    }
}

impl Display for BinaryPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StrCmp => write!(f, "StrCmp"),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum UnaryPattern {
    String,
    Array,
    Sexp,
    Boxed,
    Unboxed,
    Closure,
}

impl Display for UnaryPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String => write!(f, "String"),
            Self::Array => write!(f, "Array"),
            Self::Sexp => write!(f, "Sexp"),
            Self::Boxed => write!(f, "Boxed"),
            Self::Unboxed => write!(f, "Unboxed"),
            Self::Closure => write!(f, "Closure"),
        }
    }
}

impl UnaryPattern {
    pub fn eval(&self, scrt: Opnd) -> Opnd {
        match self {
            UnaryPattern::String => runtime::builtin::string_tag_patt(scrt),
            UnaryPattern::Array => runtime::builtin::array_tag_patt(scrt),
            UnaryPattern::Boxed => runtime::builtin::boxed_patt(scrt),
            UnaryPattern::Unboxed => runtime::builtin::unboxed_patt(scrt),
            UnaryPattern::Closure => runtime::builtin::closure_tag_patt(scrt),
            UnaryPattern::Sexp => runtime::builtin::sexp_tag_patt(scrt),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Builtin {
    Read,
    Write,
    Length,
    String,
    Array(u32),
}

impl Display for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
            Self::Length => write!(f, "length"),
            Self::String => write!(f, "string"),
            Self::Array(elts) => write!(f, ".array {elts}"),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Insn {
    Binop(Binop),
    Const(isize),
    String(StringPointer),
    Sexp {
        tag: usize,
        arity: u32,
    },
    Ld(Loc),
    Lda(Loc),
    St(Loc),
    Sti,
    Sta,
    Elem,
    Jmp(Ip),
    Cjmp(Ip, JumpCondition),
    Begin {
        args: u32,
        locals: u32,
        is_closure: bool,
    },
    End,
    Closure(Ip, Vec<Loc>),
    Callc(u32),
    Call(Ip, u32),
    CallBuiltin(Builtin),
    Ret,
    Drop,
    Dup,
    Swap,
    Tag {
        tag: usize,
        arity: u32,
    },
    Array(u32),
    Patt(Pattern),
    Fail {
        line: u32,
        col: u32,
    },
    Line(u32),
}

impl Display for Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binop(op) => write!(f, "BINOP   {op}"),
            Self::Const(val) => write!(f, "CONST   {val}"),
            Self::String(ptr) => write!(f, "STRING  {ptr}"),
            Self::Sexp { tag, arity } => write!(f, "SEXP    #{tag} {arity}"),
            Self::Ld(loc) => write!(f, "LD      {loc}"),
            Self::Lda(loc) => write!(f, "LDA     {loc}"),
            Self::St(loc) => write!(f, "ST      {loc}"),
            Self::Sti => write!(f, "STI     "),
            Self::Sta => write!(f, "STA     "),
            Self::Elem => write!(f, "ELEM    "),
            Self::Jmp(ip) => write!(f, "JMP     {ip}"),
            Self::Cjmp(ip, jc) => write!(f, "CJMP    {jc} {ip}"),
            Self::Begin {
                args,
                locals,
                is_closure: true,
            } => write!(f, "CBEGIN  A{args} L{locals}"),
            Self::Begin {
                args,
                locals,
                is_closure: false,
            } => write!(f, "BEGIN   A{args} L{locals}"),
            Self::End => write!(f, "END     "),
            Self::Closure(ip, capts) => {
                write!(f, "CLOSURE ")?;
                capts
                    .iter()
                    .map(|capt| write!(f, "{ip} {capt}"))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(())
            }
            Self::Call(ip, args) => write!(f, "CALL    {ip} {args}"),
            Self::Callc(ip) => write!(f, "CALLC   {ip}"),
            Self::CallBuiltin(builtin) => write!(f, "CALL    {builtin}"),
            Self::Ret => write!(f, "RET     "),
            Self::Drop => write!(f, "DROP    "),
            Self::Dup => write!(f, "DUP     "),
            Self::Swap => write!(f, "SWAP    "),
            Self::Tag { tag, arity } => write!(f, "TAG     #{tag} {arity}"),
            Self::Array(elts) => write!(f, "ARRAY   {elts}"),
            Self::Patt(pat) => write!(f, "PATT    {pat}"),
            Self::Fail { line, col } => write!(f, "FAIL    {line}:{col}"),
            Self::Line(line) => write!(f, "LINE    {line}"),
        }
    }
}

pub trait LoadInsn: Load {
    fn load_insn<'a>(
        &'a self,
        off: usize,
        string_tab: StringTab<'a>,
    ) -> anyhow::Result<Option<(Insn, usize)>>;
}

fn high_low(code: u8) -> (u8, u8) {
    (code >> 4, code & 0xF)
}

const BINOP_HIGH: u8 = 0;
const CONST_TO_ELEM_HIGH: u8 = 1;
const CONST_LOW: u8 = 0;
const STRING_LOW: u8 = 1;
const SEXP_LOW: u8 = 2;
const STI_LOW: u8 = 3;
const STA_LOW: u8 = 4;
const JMP_LOW: u8 = 5;
const END_LOW: u8 = 6;
const RET_LOW: u8 = 7;
const DROP_LOW: u8 = 8;
const DUP_LOW: u8 = 9;
const SWAP_LOW: u8 = 10;
const ELEM_LOW: u8 = 11;
const LD_HIGH: u8 = 2;
const LDA_HIGH: u8 = 3;
const ST_HIGH: u8 = 4;
const CJMP_TO_LINE_HIGH: u8 = 5;
const CJMP_Z_LOW: u8 = 0;
const CJMP_NZ_LOW: u8 = 1;
const BEGIN_LOW: u8 = 2;
const CBEGIN_LOW: u8 = 3;
const CLOSURE_LOW: u8 = 4;
const CALLC_LOW: u8 = 5;
const CALL_LOW: u8 = 6;
const TAG_LOW: u8 = 7;
const ARRAY_LOW: u8 = 8;
const FAIL_LOW: u8 = 9;
const LINE_LOW: u8 = 10;
const PATT_HIGH: u8 = 6;
const CALL_BUILTIN_HIGH: u8 = 7;
const END_HIGH: u8 = 15;

const GLOBAL_LOW: u8 = 0;
const LOCAL_LOW: u8 = 1;
const ARG_LOW: u8 = 2;
const CAPTURED_LOW: u8 = 3;

const BUILTINT_READ_LOW: u8 = 0;
const BUILTINT_WRITE_LOW: u8 = 1;
const BUILTINT_LENGTH_LOW: u8 = 2;
const BUILTINT_STRING_LOW: u8 = 3;
const BUILTINT_ARRAY_LOW: u8 = 4;

impl<T: Load + ?Sized> LoadInsn for T {
    fn load_insn<'a>(
        &'a self,
        off: usize,
        string_tab: StringTab<'a>,
    ) -> anyhow::Result<Option<(Insn, usize)>> {
        let load_loc = |off: usize, low: u8| match low {
            GLOBAL_LOW => Ok(Loc::Global(
                self.load_i32(off + 1)
                    .and_then(|val| Ok(val.try_into()?))
                    .context("Couldn't load Global variable location")?,
            )),
            LOCAL_LOW => Ok(Loc::Local(
                self.load_i32(off + 1)
                    .and_then(|val| Ok(val.try_into()?))
                    .context("Couldn't load Local variable location")?,
            )),
            ARG_LOW => Ok(Loc::Arg(
                self.load_i32(off + 1)
                    .and_then(|val| Ok(val.try_into()?))
                    .context("Couldn't load Arg location")?,
            )),
            CAPTURED_LOW => Ok(Loc::Captured(
                self.load_i32(off + 1)
                    .and_then(|val| Ok(val.try_into()?))
                    .context("Couldn't load Captured variable location")?,
            )),
            _ => bail!("Incorrect low opcode {low:x}"),
        };

        let (high, low) = high_low(self.load_u8(off).context("Couldn't load instruction")?);
        Ok(Some(match high {
            BINOP_HIGH => (Insn::Binop(Binop::from_low(low)?), off + 1),
            CONST_TO_ELEM_HIGH => match low {
                CONST_LOW => (Insn::Const(self.load_i32(off + 1)? as isize), off + 5),
                STRING_LOW => (Insn::String(self.load_string_pointer(off + 1)?), off + 5),
                SEXP_LOW => (
                    Insn::Sexp {
                        tag: string_tab
                            .get_tag_hash(self.load_string_pointer(off + 1).context(format!(
                                "Couldn't load string pointer in SEXP insn at {off:x}"
                            ))?)
                            .context(format!("Couldn't get hash of tag in SEXP insn at {off:x}"))?,
                        arity: self
                            .load_i32(off + 5)
                            .context(format!("Couldn't load arity count in SEXP insn at {off:x}"))?
                            .try_into()
                            .context(format!("Negative arity count in SEXP insn at {off:x}"))?,
                    },
                    off + 9,
                ),
                STI_LOW => (Insn::Sti, off + 1),
                STA_LOW => (Insn::Sta, off + 1),
                JMP_LOW => (Insn::Jmp(self.load_ip(off + 1)?), off + 5),
                END_LOW => (Insn::End, off + 1),
                RET_LOW => (Insn::Ret, off + 1),
                DROP_LOW => (Insn::Drop, off + 1),
                DUP_LOW => (Insn::Dup, off + 1),
                SWAP_LOW => (Insn::Swap, off + 1),
                ELEM_LOW => (Insn::Elem, off + 1),
                _ => bail!("Unknown low opcode {low:x} for high opcode {high:x}"),
            },
            LD_HIGH..=ST_HIGH => {
                let insn = match high {
                    LD_HIGH => {
                        Insn::Ld(load_loc(off, low).context("Couldn't load LD insn at {off:x}")?)
                    }
                    LDA_HIGH => {
                        Insn::Lda(load_loc(off, low).context("Couldn't load LDA insn at {off:x}")?)
                    }
                    ST_HIGH => {
                        Insn::St(load_loc(off, low).context("Couldn't load ST insn at {off:x}")?)
                    }
                    _ => unreachable!(),
                };
                (insn, off + 5)
            }
            CJMP_TO_LINE_HIGH => match low {
                CJMP_Z_LOW => (
                    Insn::Cjmp(self.load_ip(off + 1)?, JumpCondition::Zero),
                    off + 5,
                ),
                CJMP_NZ_LOW => (
                    Insn::Cjmp(self.load_ip(off + 1)?, JumpCondition::NotZero),
                    off + 5,
                ),
                BEGIN_LOW..=CBEGIN_LOW => {
                    let args: u32 = self
                        .load_i32(off + 1)?
                        .try_into()
                        .context("Negative args count in BEGIN insn")?;
                    let locals: u32 = self
                        .load_i32(off + 5)?
                        .try_into()
                        .context("Negative args count in BEGIN insn")?;
                    let is_closure = high == CBEGIN_LOW;
                    (
                        Insn::Begin {
                            args,
                            locals,
                            is_closure,
                        },
                        off + 9,
                    )
                }
                CLOSURE_LOW => {
                    let ip = self.load_ip(off + 1)?;
                    let ncapts: u32 = self
                        .load_i32(off + 5)?
                        .try_into()
                        .context("Negative closure captures count in CLOSURE insn")?;
                    let capts = (0..ncapts as usize)
                        .map(|i| {
                            let low = self
                                .load_u8(off + 9 + i * 5)
                                .context("Couldn't load loc type in CLOSURE insn")?;
                            load_loc(off + 9 + i * 5, low)
                        })
                        .collect::<anyhow::Result<Vec<Loc>>>()?;
                    (Insn::Closure(ip, capts), off + 9 + ncapts as usize * 5)
                }
                CALLC_LOW => (
                    Insn::Callc(
                        self.load_i32(off + 1)
                            .context(format!("Couldn't load ip in CALLC insn at {off:x}"))?
                            .try_into()?,
                    ),
                    off + 5,
                ),
                CALL_LOW => (
                    Insn::Call(
                        self.load_ip(off + 1)
                            .context(format!("Couldn't load ip in CALL insn at {off:x}"))?,
                        self.load_i32(off + 5)
                            .context(format!("Couldn't load arg count in CALL insn at {off:x}"))?
                            .try_into()
                            .context(format!("Negative arg count in CALL insn at {off:x}"))?,
                    ),
                    off + 9,
                ),
                TAG_LOW => (
                    Insn::Tag {
                        tag: string_tab
                            .get_tag_hash(self.load_string_pointer(off + 1).context(format!(
                                "Couldn't load string pointer in TAG insn at {off:x}"
                            ))?)
                            .context(format!("Couldn't get hash of tag in TAG insn at {off:x}"))?,
                        arity: self
                            .load_i32(off + 5)
                            .context(format!("Couldn't load arity count in TAG insn at {off:x}"))?
                            .try_into()
                            .context(format!("Negative arity count in TAG insn at {off:x}"))?,
                    },
                    off + 9,
                ),
                ARRAY_LOW => (
                    Insn::Array(
                        self.load_i32(off + 1)
                            .context(format!("Couldn't load length in ARRAY insn at {off:x}"))?
                            .try_into()
                            .context(format!("Negative array length in ARRAY insn at {off:x}"))?,
                    ),
                    off + 5,
                ),
                FAIL_LOW => (
                    Insn::Fail {
                        line: self
                            .load_i32(off + 1)
                            .context(format!("Couldn't load line in FAIL insn at {off:x}"))?
                            .try_into()
                            .context(format!("Negative line in FAIL insn at {off:x}"))?,
                        col: self
                            .load_i32(off + 1)
                            .context(format!("Couldn't load col in FAIL insn at {off:x}"))?
                            .try_into()
                            .context(format!("Negative col in FAIL insn at {off:x}"))?,
                    },
                    off + 9,
                ),
                LINE_LOW => (
                    Insn::Line(
                        self.load_i32(off + 1)
                            .context(format!("Couldn't load line in LINE insn at {off:x}"))?
                            .try_into()
                            .context(format!("Negative line in LINE insn at {off:x}"))?,
                    ),
                    off + 5,
                ),
                _ => bail!("Incorrect low opcode {low:x} for high opcode {high:x}"),
            },
            PATT_HIGH => (Insn::Patt(Pattern::from_low(low)?), off + 1),
            CALL_BUILTIN_HIGH => match low {
                BUILTINT_READ_LOW => (Insn::CallBuiltin(Builtin::Read), off + 1),
                BUILTINT_WRITE_LOW => (Insn::CallBuiltin(Builtin::Write), off + 1),
                BUILTINT_LENGTH_LOW => (Insn::CallBuiltin(Builtin::Length), off + 1),
                BUILTINT_STRING_LOW => (Insn::CallBuiltin(Builtin::String), off + 1),
                BUILTINT_ARRAY_LOW => (
                    Insn::CallBuiltin(Builtin::Array(
                        self.load_i32(off + 1)
                            .context(format!(
                                "Couldn't load length in CALL .array insn at {off:x}"
                            ))?
                            .try_into()
                            .context(format!(
                                "Negative array length in CALL .array insn at {off:x}"
                            ))?,
                    )),
                    off + 5,
                ),
                _ => bail!("Incorrect low opcode {low:x} for high opcode {high:x}"),
            },
            END_HIGH => return Ok(None),
            _ => bail!("Unknown high opcode"),
        }))
    }
}

#[derive(Clone)]
pub struct AnalyticInsn {
    addr: usize,
    insn: Insn,
}

impl AnalyticInsn {
    pub fn new(insn: Insn, addr: usize) -> Self {
        Self { addr, insn }
    }
}

impl Display for AnalyticInsn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.insn)
    }
}

impl Hash for AnalyticInsn {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.insn.hash(state);
    }
}

impl PartialEq for AnalyticInsn {
    fn eq(&self, other: &Self) -> bool {
        self.insn == other.insn
    }
}

impl Eq for AnalyticInsn {}

impl crate::analyze::insn::Instruction for AnalyticInsn {
    fn addr(&self) -> usize {
        self.addr
    }

    fn jumps<'a>(&'a self) -> Box<dyn Iterator<Item = usize> + 'a> {
        match self.insn.clone() {
            Insn::Jmp(ip) => Box::new(iter::once(ip.0)),
            Insn::Cjmp(ip, _) => Box::new(iter::once(ip.0)),
            Insn::Call(ip, _) => Box::new(iter::once(ip.0)),
            Insn::Line(_) => panic!("Labels should not be analytic insns"),
            _ => Box::new(iter::empty()),
        }
    }

    fn is_return(&self) -> bool {
        match self.insn.clone() {
            Insn::Jmp(..) => true,
            Insn::Cjmp(..) => true,
            Insn::Begin { .. } => true,
            Insn::End => true,
            Insn::Callc(..) => true,
            Insn::Call(..) => true,
            Insn::CallBuiltin(..) => true,
            Insn::Ret => true,
            Insn::Line(..) => panic!("Labels should not be analytic insns"),
            _ => false,
        }
    }
}
