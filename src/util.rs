use anyhow::anyhow;

const CHARS: &[u8] = b"_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'";

const BITS_PER_CHAR: usize = 6;

pub fn hash(data: &[u8]) -> anyhow::Result<usize> {
    data.iter().take(10).try_fold(0usize, |cum, cur| {
        CHARS
            .iter()
            .enumerate()
            .find(|(_, c)| **c == *cur)
            .ok_or(anyhow!("Incorrect sexp tag symbol"))
            .map(|val| (cum << BITS_PER_CHAR) | val.0)
    })
}

macro_rules! catch {
    ($b:block) => {
        (|| $b)()
    };
}

pub(crate) use catch;
