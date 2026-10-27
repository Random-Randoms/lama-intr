use crate::{
    closure::Closure,
    ctrl_stack::Stack,
    opnd::{Opnd, Value},
    program::{
        Ip, Program,
        insn::{Builtin, Insn, Pattern},
    },
    runtime,
};

use anyhow::Context;
use log::debug;

pub struct Interpreter<'a> {
    ip: Ip,
    ctrl_stack: Stack,
    program: Program<'a>,
}

impl<'a> Interpreter<'a> {
    pub fn new(program: Program<'a>) -> anyhow::Result<Self> {
        let ip = program.entry_point.code_off;
        let main = program.code.get_fun(ip)?;
        let mut ctrl_stack = Stack::new(program.global_area_elts, 1 << 15)?;
        ctrl_stack.push_frame(None, main.locals as usize, vec![], ip)?;
        Ok(Self {
            ip,
            ctrl_stack,
            program,
        })
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        runtime::gc::init();

        'a: loop {
            let (insn, next) = self.program.code.get_insn(self.ip)?.clone();
            debug!("Execute {insn}");
            let mut next_ip = Some(next);
            match insn.clone() {
                Insn::Const(val) => {
                    self.ctrl_stack
                        .cur_frame()?
                        .push(Value::Int(val).into_opnd())?;
                    Ok::<(), anyhow::Error>(())
                }
                Insn::Drop => {
                    self.ctrl_stack.cur_frame()?.pop()?;
                    Ok(())
                }
                Insn::Dup => {
                    let val = self.ctrl_stack.cur_frame()?.pop()?;
                    self.ctrl_stack.cur_frame()?.push(val)?;
                    self.ctrl_stack.cur_frame()?.push(val)?;
                    Ok(())
                }
                Insn::Swap => {
                    let top = self.ctrl_stack.cur_frame()?.pop()?;
                    let bot = self.ctrl_stack.cur_frame()?.pop()?;
                    self.ctrl_stack.cur_frame()?.push(top)?;
                    self.ctrl_stack.cur_frame()?.push(bot)?;
                    Ok(())
                }
                Insn::Binop(op) => {
                    let rhs = self.ctrl_stack.cur_frame()?.pop()?;
                    let lhs = self.ctrl_stack.cur_frame()?.pop()?;
                    self.ctrl_stack.cur_frame()?.push(op.eval(lhs, rhs)?)?;
                    Ok(())
                }
                Insn::String(ptr) => {
                    let opnd = runtime::builtin::string(self.program.string_tab.get_ptr(ptr)?);
                    self.ctrl_stack.cur_frame()?.push(opnd)?;
                    Ok(())
                }
                Insn::Sexp { tag, arity } => {
                    let args = self
                        .ctrl_stack
                        .cur_frame()?
                        .pop_n(arity)?
                        .map(|opnd| opnd.raw())
                        .collect::<Vec<_>>();
                    debug!("{args:?}");
                    let opnd = runtime::builtin::sexp(args, tag);
                    self.ctrl_stack.cur_frame()?.push(opnd)?;
                    Ok(())
                }
                Insn::Elem => {
                    let arr = self.ctrl_stack.cur_frame()?.pop()?;
                    let ind = self.ctrl_stack.cur_frame()?.pop()?;
                    let opnd = runtime::builtin::elem(ind, arr);
                    self.ctrl_stack.cur_frame()?.push(opnd)?;
                    Ok(())
                }
                Insn::Ld(loc) => {
                    let val = *self.ctrl_stack.cur_frame()?.get(loc)?;
                    //println!("loaded: {val:?}");
                    self.ctrl_stack.cur_frame()?.push(val)?;
                    Ok(())
                }
                Insn::Lda(_) => panic!("Lda unsupported!"),
                Insn::St(loc) => {
                    let val = self.ctrl_stack.cur_frame()?.pop()?;
                    *self.ctrl_stack.cur_frame()?.get_mut(loc)? = val;
                    self.ctrl_stack.cur_frame()?.push(val)?;
                    Ok(())
                }
                Insn::Sti => panic!("unsupported!"),
                Insn::Sta => {
                    let val = self.ctrl_stack.cur_frame()?.pop()?;
                    let ind = self.ctrl_stack.cur_frame()?.pop()?;
                    let obj = self.ctrl_stack.cur_frame()?.pop()?;
                    let ret = runtime::builtin::sta(obj, ind, val);
                    self.ctrl_stack.cur_frame()?.push(ret)?;
                    Ok(())
                }
                Insn::Jmp(ip) => {
                    next_ip = Some(ip);
                    Ok(())
                }
                Insn::Cjmp(ip, cond) => {
                    let val = self.ctrl_stack.cur_frame()?.pop()?;
                    if cond.eval(val)? {
                        next_ip = Some(ip);
                    }
                    Ok(())
                }
                Insn::Closure(ip, capts) => {
                    let mut args = vec![ip.into_opnd()];
                    let nargs = capts.len() + 1;
                    capts
                        .into_iter()
                        .map(|capt| {
                            args.push(*(self.ctrl_stack.cur_frame()?.get(capt)?));
                            Ok::<_, anyhow::Error>(())
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    //println!("args: {args:?}");
                    let res = runtime::builtin::closure(args, nargs as u32);
                    self.ctrl_stack.cur_frame()?.push(res)?;
                    Ok(())
                }
                Insn::Callc(args) => {
                    let argvs = self
                        .ctrl_stack
                        .cur_frame()?
                        .pop_n(args)?
                        .collect::<Vec<_>>();
                    let closure = self.ctrl_stack.cur_frame()?.pop()?;
                    let ip = unsafe { closure.get_ip()? };
                    let ret = next_ip.expect("Is some before insn execuion");
                    let locals = self.program.code.get_fun(ip)?.locals as usize;
                    self.ctrl_stack
                        .push_frame(Some(closure), locals, argvs, ret)?;
                    next_ip = Some(ip);
                    Ok(())
                }
                Insn::Call(ip, args) => {
                    let argvs = self
                        .ctrl_stack
                        .cur_frame()?
                        .pop_n(args)?
                        .collect::<Vec<_>>();
                    let ret = next_ip.expect("Is some before insn execuion");
                    let locals = self.program.code.get_fun(ip)?.locals as usize;
                    self.ctrl_stack.push_frame(None, locals, argvs, ret)?;
                    next_ip = Some(ip);
                    Ok(())
                }
                Insn::CallBuiltin(builtin) => {
                    match builtin {
                        Builtin::Read => {
                            let res = runtime::builtin::read();
                            self.ctrl_stack.cur_frame()?.push(res)?;
                        }
                        Builtin::Write => {
                            let arg = self.ctrl_stack.cur_frame()?.pop()?;
                            runtime::builtin::write(arg);
                            self.ctrl_stack.cur_frame()?.push(Opnd::null())?;
                        }
                        Builtin::Length => {
                            let arg = self.ctrl_stack.cur_frame()?.pop()?;
                            let res = runtime::builtin::length(arg);
                            self.ctrl_stack.cur_frame()?.push(res)?;
                        }
                        Builtin::String => {
                            let arg = self.ctrl_stack.cur_frame()?.pop()?;
                            let res = runtime::builtin::l_string(arg);
                            self.ctrl_stack.cur_frame()?.push(res)?;
                        }
                        Builtin::Array(nargs) => {
                            let args = self
                                .ctrl_stack
                                .cur_frame()?
                                .pop_n(nargs)?
                                .collect::<Vec<_>>();
                            let res = runtime::builtin::array(args, nargs);
                            self.ctrl_stack.cur_frame()?.push(res)?;
                        }
                    }
                    Ok(())
                }
                Insn::End => {
                    let res = self.ctrl_stack.cur_frame()?.pop()?;
                    let ret = self.ctrl_stack.pop_frame()?;
                    next_ip = ret;
                    if next_ip.is_some() {
                        self.ctrl_stack.cur_frame()?.push(res)?;
                    }
                    Ok(())
                }
                Insn::Ret => {
                    let res = self.ctrl_stack.cur_frame()?.pop()?;
                    let ret = self.ctrl_stack.pop_frame()?;
                    next_ip = ret;
                    if next_ip.is_some() {
                        self.ctrl_stack.cur_frame()?.push(res)?;
                    }
                    Ok(())
                }
                Insn::Tag { tag, arity } => {
                    let scrt = self.ctrl_stack.cur_frame()?.pop()?;
                    let result = runtime::builtin::tag(scrt, tag, arity);
                    self.ctrl_stack.cur_frame()?.push(result)?;
                    Ok(())
                }
                Insn::Array(len) => {
                    let scrt = self.ctrl_stack.cur_frame()?.pop()?;
                    let result = runtime::builtin::array_patt(scrt, len);
                    self.ctrl_stack.cur_frame()?.push(result)?;
                    Ok(())
                }
                Insn::Patt(patt) => {
                    let result = match patt {
                        Pattern::Binary(bin) => {
                            let rhs = self.ctrl_stack.cur_frame()?.pop()?;
                            let lhs = self.ctrl_stack.cur_frame()?.pop()?;
                            bin.eval(lhs, rhs)
                        }
                        Pattern::Unary(un) => {
                            let scrt = self.ctrl_stack.cur_frame()?.pop()?;
                            un.eval(scrt)
                        }
                    };
                    self.ctrl_stack.cur_frame()?.push(result)?;
                    Ok(())
                }
                Insn::Fail { line, col } => {
                    let scrt = self.ctrl_stack.cur_frame()?.pop()?;
                    runtime::builtin::match_failure(scrt, "baka", line, col);
                    Ok(())
                }
                Insn::Line(_) => Ok(()),
                Insn::Begin { .. } => Ok(()),
            }
            .context(format!("Executing {insn}, at ip {}", self.ip))?;
            match next_ip {
                Some(ip) => {
                    self.ip = ip;
                }
                None => {
                    break 'a Ok(());
                }
            }
        }
    }
}
