use crate::{opnd::Opnd, program::Ip};
use std::ops::Add;

use anyhow::Context;

/// # Safety
///
/// Caller must ensure that ind is a correct index for closure
pub unsafe trait Closure<'a> {
    unsafe fn get_ip(self) -> anyhow::Result<Ip>;

    unsafe fn get(self, ind: usize) -> anyhow::Result<&'a Opnd>;
}

/// # Safety
///
/// Caller must ensure that ind is a correct index for closure
pub unsafe trait ClosureMut<'a>: Closure<'a> {
    unsafe fn get_mut(self, ind: usize) -> anyhow::Result<&'a mut Opnd>;
}

unsafe impl<'a> Closure<'a> for &'a mut Opnd {
    unsafe fn get_ip(self) -> anyhow::Result<Ip> {
        unsafe {
            Ok(*(self
                .into_value()
                .into_ptr()
                .context("Closure was boxed value")? as *const Ip))
        }
    }

    unsafe fn get(self, ind: usize) -> anyhow::Result<&'a Opnd> {
        unsafe {
            (self
                .into_value()
                .into_ptr()
                .context("Closure was boxed value")?
                .add(8 * (ind + 1)) as *const Opnd)
                .as_ref()
                .context("Couldn't convert pointer to ref")
        }
    }
}

unsafe impl<'a> Closure<'a> for &'a Opnd {
    unsafe fn get_ip(self) -> anyhow::Result<Ip> {
        unsafe {
            Ok(*(self
                .into_value()
                .into_ptr()
                .context("Closure was boxed value")? as *const Ip))
        }
    }

    unsafe fn get(self, ind: usize) -> anyhow::Result<&'a Opnd> {
        unsafe {
            (self
                .into_value()
                .into_ptr()
                .context("Closure was boxed value")?
                .add(8 * (ind + 1)) as *const Opnd)
                .as_ref()
                .context("Couldn't convert pointer to ref")
        }
    }
}

unsafe impl<'a> ClosureMut<'a> for &'a mut Opnd {
    unsafe fn get_mut(self, ind: usize) -> anyhow::Result<&'a mut Opnd> {
        unsafe {
            (self
                .into_value()
                .into_ptr()
                .context("Closure was boxed value")?
                .add(8 * (ind + 1)) as *mut Opnd)
                .as_mut()
                .context("Couldn't convert pointer to ref")
        }
    }
}
