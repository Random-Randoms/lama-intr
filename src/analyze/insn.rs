use std::{fmt::Display, hash::Hash};

pub trait Instruction: Eq + Hash + Display + Clone {
    fn addr(&self) -> usize;

    fn jumps<'a>(&'a self) -> Box<dyn Iterator<Item = usize> + 'a>;

    fn is_return(&self) -> bool;
}

pub trait Code<'a, T: Instruction> {
    fn instructions(&'a self) -> Box<dyn Iterator<Item = T> + 'a>;

    fn entry_points(&'a self) -> Box<dyn Iterator<Item = usize> + 'a>;
}
