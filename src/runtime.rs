use std::ffi::c_void;

use libc::{c_char, c_int, size_t};

pub type Obj = *const c_void;
pub type Objs = *const Obj;
pub type Cstr = *const c_char;
pub type Aint = i64;
pub type Aints = *const Aint;

#[link(name = "runtime")]
unsafe extern "C" {
    fn __gc_init();

    fn Bstring(val: *const Cstr) -> Obj;

    fn Bsexp(args: Objs, nargs: Aint) -> Obj;

    fn Barray(args: Aints, nargs: Aint) -> Obj;

    fn Bclosure(args: Aints, nargs: Aint) -> Obj;

    fn Belem(arr: Obj, ind: Aint) -> Obj;

    fn Bsta(obj: Obj, ind: Aint, val: Obj) -> Obj;

    fn Btag(obj: Obj, tag: Aint, arity: Aint) -> Obj;

    fn Barray_patt(scrt: Obj, len: Aint) -> Obj;

    fn Bstring_patt(lhs: Obj, rhs: Obj) -> Obj;

    fn Barray_tag_patt(scrt: Obj) -> Obj;

    fn Bstring_tag_patt(scrt: Obj) -> Obj;

    fn Bsexp_tag_patt(scrt: Obj) -> Obj;

    fn Bclosure_tag_patt(scrt: Obj) -> Obj;

    fn Bboxed_patt(scrt: Obj) -> Obj;

    fn Bunboxed_patt(scrt: Obj) -> Obj;

    fn Bmatch_failure(scrt: Obj, fname: Cstr, line: Aint, col: Aint);

    fn Lwrite(n: Aint) -> c_int;

    fn Lread() -> c_int;

    fn Llength(obj: Obj) -> Aint;

    fn Lstring(obj: *const Aint) -> Obj;

    fn set_stack_borders(top: size_t, bottom: size_t);
}

pub mod gc {
    use super::{__gc_init, set_stack_borders};
    use crate::opnd::Opnd;

    pub fn init() {
        unsafe {
            __gc_init();
        }
    }

    pub fn set_borders(stack: &[Opnd]) {
        unsafe {
            let top = stack.as_ptr() as usize;
            let bottom = (stack.as_ptr().add(stack.len())) as usize;
            set_stack_borders(top, bottom);
        }
    }
}

pub mod builtin {
    use super::*;
    use crate::opnd::{Opnd, Value};

    pub fn string(ptr: usize) -> Opnd {
        Opnd::from_raw(unsafe { Bstring((&ptr as *const usize).cast()) as isize })
    }

    pub fn sexp(mut args: Vec<isize>, tag: usize) -> Opnd {
        args.push(Value::Int(tag as isize).into_opnd().raw());
        Value::Ptr(unsafe {
            Bsexp(
                args.as_ptr().cast(),
                Value::Int(args.len() as isize).into_opnd().raw() as Aint,
            ) as usize
        })
        .into_opnd()
    }

    pub fn array(args: Vec<Opnd>, nargs: u32) -> Opnd {
        Opnd::from_raw(unsafe {
            Barray(
                args.as_ptr().cast(),
                Value::Int(nargs as isize).into_opnd().raw() as Aint,
            ) as isize
        })
    }

    pub fn closure(args: Vec<Opnd>, nargs: u32) -> Opnd {
        let nargs = Value::Int(nargs as isize).into_opnd();
        Opnd::from_raw(unsafe { Bclosure(args.as_ptr().cast(), nargs.raw() as Aint) as isize })
    }

    pub fn elem(ptr: Opnd, ind: Opnd) -> Opnd {
        Opnd::from_raw(unsafe { Belem(ptr.raw() as Obj, ind.raw() as Aint) as isize })
    }

    pub fn sta(obj: Opnd, ind: Opnd, val: Opnd) -> Opnd {
        Opnd::from_raw(unsafe {
            Bsta(obj.raw() as Obj, ind.raw() as Aint, val.raw() as Obj) as isize
        })
    }

    pub fn tag(scrt: Opnd, tag: usize, arity: u32) -> Opnd {
        let tag = Value::Int(tag as isize).into_opnd();
        let arity = Value::Int(arity as isize).into_opnd();
        Opnd::from_raw(unsafe {
            Btag(scrt.raw() as Obj, tag.raw() as Aint, arity.raw() as Aint) as isize
        })
    }

    pub fn array_patt(scrt: Opnd, len: u32) -> Opnd {
        let len = Value::Int(len as isize).into_opnd();
        Opnd::from_raw(unsafe { Barray_patt(scrt.raw() as Obj, len.raw() as Aint) as isize })
    }

    pub fn string_patt(lhs: Opnd, rhs: Opnd) -> Opnd {
        Opnd::from_raw(unsafe { Bstring_patt(lhs.raw() as Obj, rhs.raw() as Obj) as isize })
    }

    pub fn array_tag_patt(scrt: Opnd) -> Opnd {
        Opnd::from_raw(unsafe { Barray_tag_patt(scrt.raw() as Obj) as isize })
    }

    pub fn string_tag_patt(scrt: Opnd) -> Opnd {
        Opnd::from_raw(unsafe { Bstring_tag_patt(scrt.raw() as Obj) as isize })
    }

    pub fn sexp_tag_patt(scrt: Opnd) -> Opnd {
        Opnd::from_raw(unsafe { Bsexp_tag_patt(scrt.raw() as Obj) as isize })
    }

    pub fn closure_tag_patt(scrt: Opnd) -> Opnd {
        Opnd::from_raw(unsafe { Bclosure_tag_patt(scrt.raw() as Obj) as isize })
    }

    pub fn boxed_patt(scrt: Opnd) -> Opnd {
        Opnd::from_raw(unsafe { Bboxed_patt(scrt.raw() as Obj) as isize })
    }

    pub fn unboxed_patt(scrt: Opnd) -> Opnd {
        Opnd::from_raw(unsafe { Bunboxed_patt(scrt.raw() as Obj) as isize })
    }

    pub fn match_failure(scrt: Opnd, fname: &str, line: u32, col: u32) {
        unsafe {
            Bmatch_failure(
                scrt.raw() as Obj,
                fname.as_ptr().cast(),
                line as Aint,
                col as Aint,
            );
        }
    }

    pub fn read() -> Opnd {
        Opnd::from_raw(unsafe { Lread() as isize })
    }

    pub fn write(opnd: Opnd) {
        unsafe {
            Lwrite(opnd.raw() as Aint);
        }
    }

    pub fn length(obj: Opnd) -> Opnd {
        Opnd::from_raw(unsafe { Llength(obj.raw() as Obj) as isize })
    }

    pub fn l_string(obj: Opnd) -> Opnd {
        let obj = obj.raw();
        Opnd::from_raw(unsafe { Lstring((&obj as *const isize).cast()) as isize })
    }
}
