use anyhow::{Context, anyhow};

pub trait Load {
    fn subslice(&self, off: usize, len: usize) -> anyhow::Result<&[u8]>;

    fn load_i32(&self, off: usize) -> anyhow::Result<i32>;

    fn load_usize(&self, off: usize) -> anyhow::Result<usize>;

    fn load_u8(&self, off: usize) -> anyhow::Result<u8>;
}

impl Load for [u8] {
    fn subslice(&self, off: usize, len: usize) -> anyhow::Result<&[u8]> {
        self.get(off..off + len).context(format!(
            "Slice ({:x} .. {:x}) overflows file size ({:x})",
            off,
            off + len,
            self.len()
        ))
    }

    fn load_i32(&self, off: usize) -> anyhow::Result<i32> {
        Ok(self
            .get(off..off + 4)
            .context(format!(
                "Int value (at {:x}) overflows file size ({:x})",
                off,
                self.len()
            ))?
            .as_array()
            .map(|arr| i32::from_le_bytes(*arr))
            .expect("Exactly 4 bytes slice to array"))
    }

    fn load_usize(&self, off: usize) -> anyhow::Result<usize> {
        Ok(self
            .get(off..off + 8)
            .context(format!(
                "Pointer value (at {:x}) overflows file size ({:x})",
                off,
                self.len()
            ))?
            .as_array()
            .map(|arr| usize::from_le_bytes(*arr))
            .expect("Exactly 8 bytes slice to array"))
    }

    fn load_u8(&self, off: usize) -> anyhow::Result<u8> {
        self.get(off)
            .ok_or(anyhow!(
                "Byte value (at {:x}) overflows file size ({:x})",
                off,
                self.len()
            ))
            .cloned()
    }
}
