use std::iter;

use anyhow::{Context, anyhow, bail};
use log::debug;

use crate::{
    closure::{Closure, ClosureMut},
    opnd::Opnd,
    program::{Ip, loc::Loc},
    runtime,
};

struct Frame {
    ret: Ip,
    is_closure: bool,
    sp: usize,
    locals: usize,
    args: usize,
}

impl Frame {
    fn closure_off(&self) -> usize {
        if self.is_closure { 1 } else { 0 }
    }

    fn closure(&self) -> anyhow::Result<usize> {
        if self.is_closure {
            Ok(self.sp)
        } else {
            bail!("Frame is not a closure")
        }
    }

    fn local(&self, ind: usize) -> anyhow::Result<usize> {
        if ind >= self.locals {
            bail!("Local index {ind} out of range for {} locals", self.locals)
        }
        Ok(self.sp + self.closure_off() + ind)
    }

    fn arg(&self, ind: usize) -> anyhow::Result<usize> {
        if ind >= self.args {
            bail!("Argument index {ind} out of range for {} args", self.args)
        }
        Ok(self.sp + self.closure_off() + self.locals + ind)
    }

    fn total(&self) -> usize {
        self.closure_off() + self.locals + self.args
    }

    fn end(&self) -> usize {
        self.sp + self.total()
    }
}

pub struct CurFrame<'a> {
    frame: &'a Frame,
    op_stack: &'a mut OpStack,
}

impl<'a> CurFrame<'a> {
    pub fn push(&'a mut self, opnd: Opnd) -> anyhow::Result<()> {
        self.op_stack.push(opnd)
    }

    pub fn pop(&mut self) -> anyhow::Result<Opnd> {
        if self.op_stack.len() <= self.frame.end() {
            bail!("Operand stack depleted: trying to pop frame data")
        }
        Ok(self.op_stack.pop().expect("Should have elements"))
    }

    pub fn pop_n(&'a mut self, n: u32) -> anyhow::Result<impl DoubleEndedIterator<Item = Opnd>> {
        if self.op_stack.len() - self.frame.end() < (n as usize) {
            bail!(
                "Operand stack depleted: trying to pop {n} elemetns, but only {} left",
                self.op_stack.len() - self.frame.end()
            )
        }
        Ok(self
            .op_stack
            .split_off(self.op_stack.len() - n as usize)?
            .into_iter())
    }

    pub fn get(&'a self, loc: Loc) -> anyhow::Result<&'a Opnd> {
        match loc {
            Loc::Local(ind) => self
                .op_stack
                .get(self.frame.local(ind as usize)?)
                .context(format!("Incorrect local index {ind}")),
            Loc::Global(ind) => self
                .op_stack
                .get(ind as usize)
                .context(format!("Incorrect global pointer {ind}")),
            Loc::Captured(ind) => unsafe {
                self.op_stack
                    .get(self.frame.closure()?)
                    .expect("Closure should be on stack")
                    .get(ind as usize)
            },
            Loc::Arg(ind) => self
                .op_stack
                .get(self.frame.arg(ind as usize)?)
                .context(format!("Incorrect argument index {ind}")),
        }
        .context("Loading {loc}")
    }

    pub fn get_mut(&'a mut self, loc: Loc) -> anyhow::Result<&'a mut Opnd> {
        match loc {
            Loc::Local(ind) => self
                .op_stack
                .get_mut(self.frame.local(ind as usize)?)
                .context(format!("Incorrect local index {ind}")),
            Loc::Global(ind) => self
                .op_stack
                .get_mut(ind as usize)
                .context(format!("Incorrect global pointer {ind}")),
            Loc::Captured(ind) => unsafe {
                self.op_stack
                    .get_mut(self.frame.closure()?)
                    .expect("Closure should be on stack")
                    .get_mut(ind as usize)
            },
            Loc::Arg(ind) => self
                .op_stack
                .get_mut(self.frame.arg(ind as usize)?)
                .context(format!("Incorrect argument index {ind}")),
        }
        .context("Loading {loc}")
    }

    fn validate(&self) -> anyhow::Result<()> {
        if self.frame.end() != self.op_stack.len() {
            bail!("Incorrect stack state when popping stackframe")
        }
        Ok(())
    }
}

const MAX_OPNDS: usize = 1 << 14;

#[repr(align(16))]
struct Wrap([Opnd; MAX_OPNDS]);

#[repr(C)]
struct OpStack {
    opnds: Wrap,
    ptr: usize,
}

impl OpStack {
    fn new(globals: usize) -> anyhow::Result<Self> {
        if globals > MAX_OPNDS {
            bail!("Globals ({globals}) overflow operand stack {MAX_OPNDS}");
        }
        let slf = Self {
            opnds: Wrap([Opnd::null(); MAX_OPNDS]),
            ptr: globals,
        };
        slf.set_borders();
        Ok(slf)
    }

    fn set_borders(&self) {
        runtime::gc::set_borders(self.opnds.0.get(..self.ptr).expect("should be valid range"));
    }

    fn get(&self, ind: usize) -> anyhow::Result<&Opnd> {
        self.opnds.0.get(ind).ok_or(anyhow!(
            "Cannot access {ind}'th element of {} element operand stack",
            self.ptr
        ))
    }

    fn get_mut(&mut self, ind: usize) -> anyhow::Result<&mut Opnd> {
        self.opnds.0.get_mut(ind).ok_or(anyhow!(
            "Cannot access {ind}'th element of {} element operand stack",
            self.ptr
        ))
    }

    fn len(&self) -> usize {
        self.ptr
    }

    fn push(&mut self, opnd: Opnd) -> anyhow::Result<()> {
        if self.ptr == MAX_OPNDS {
            bail!("Operand stack overflow");
        }
        self.opnds.0[self.ptr] = opnd;
        self.ptr += 1;
        self.set_borders();
        Ok(())
    }

    fn pop(&mut self) -> anyhow::Result<Opnd> {
        if self.ptr == 0 {
            bail!("Operand stack depletion");
        }
        self.ptr -= 1;
        self.set_borders();
        Ok(self.opnds.0[self.ptr])
    }

    fn split_off(&mut self, off: usize) -> anyhow::Result<Vec<Opnd>> {
        let res = Vec::from_iter(
            self.opnds
                .0
                .get(off..self.ptr)
                .ok_or(anyhow!(
                    "Cannot split operand stack of length {} from element {off}",
                    self.ptr
                ))?
                .iter()
                .copied(),
        );
        self.ptr = off;
        self.set_borders();
        Ok(res)
    }
}

pub struct Stack {
    op_stack: OpStack,
    frames: Vec<Frame>,
}

impl Stack {
    pub fn new(globals: usize, max_frames: usize) -> anyhow::Result<Self> {
        {
            Ok::<_, anyhow::Error>(Self {
                op_stack: OpStack::new(globals)?,
                frames: Vec::with_capacity(max_frames),
            })
        }
        .context("Failed to create program stack")
    }

    pub fn push_frame(
        &mut self,
        closure: Option<Opnd>,
        locals: usize,
        args: Vec<Opnd>,
        ret: Ip,
    ) -> anyhow::Result<()> {
        let frame = Frame {
            ret,
            is_closure: closure.is_some(),
            sp: self.op_stack.len(),
            locals,
            args: args.len(),
        };
        if self.frames.len() == self.frames.capacity() {
            bail!("Ctrl stack overflow")
        }
        closure
            .into_iter()
            .chain(iter::repeat_n(Opnd::null(), locals).chain(args))
            .map(|opnd| self.op_stack.push(opnd))
            .collect::<anyhow::Result<Vec<_>, _>>()?;
        self.frames.push(frame);
        debug!("Push frame. Now {} frames", self.frames.len());
        Ok(())
    }

    pub fn pop_frame(&mut self) -> anyhow::Result<Option<Ip>> {
        self.cur_frame()?.validate()?;
        let frame = self.frames.pop().ok_or(anyhow!("Ctrl stack depleted"))?;
        debug!("Pop frame. {} left", self.frames.len());
        (0..frame.total()).for_each(|_| {
            self.op_stack
                .pop()
                .expect("Should not fail after validation");
        });
        Ok(if self.frames.is_empty() {
            None
        } else {
            Some(frame.ret)
        })
    }

    pub fn cur_frame<'a>(&'a mut self) -> anyhow::Result<CurFrame<'a>> {
        Ok(CurFrame {
            frame: self.frames.last().ok_or(anyhow!("Ctrl stack depleted"))?,
            op_stack: &mut self.op_stack,
        })
    }
}
