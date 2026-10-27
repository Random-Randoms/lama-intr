use anyhow::anyhow;

#[derive(Debug, Default, Clone, Copy)]
#[repr(align(8))]
pub struct Opnd(isize);

pub enum Value {
    Int(isize),
    Ptr(usize),
}

impl Value {
    pub fn into_opnd(self) -> Opnd {
        match self {
            Value::Int(val) => Opnd(val * 2 + 1),
            Value::Ptr(val) => Opnd(val as isize),
        }
    }

    pub fn into_int(self) -> anyhow::Result<isize> {
        match self {
            Value::Int(val) => Ok(val),
            _ => Err(anyhow!("Expected unboxed value")),
        }
    }

    pub fn into_ptr(self) -> anyhow::Result<usize> {
        match self {
            Value::Ptr(val) => Ok(val),
            _ => Err(anyhow!("Expected pointer")),
        }
    }
}

impl Opnd {
    pub fn null() -> Self {
        Self(0)
    }

    pub fn from_raw(val: isize) -> Self {
        Self(val)
    }

    pub fn raw(self) -> isize {
        self.0
    }

    pub fn into_value(self) -> Value {
        if self.0 % 2 != 0 {
            Value::Int((self.0 - 1) / 2)
        } else {
            Value::Ptr(self.0 as usize)
        }
    }
}
