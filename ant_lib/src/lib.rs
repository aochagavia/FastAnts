#![feature(conservative_impl_trait, inclusive_range_syntax, slice_patterns, pub_restricted)]

mod ant;
mod instruction;
mod simulator;
pub mod test_data;
mod util;
mod world;

#[cfg(test)]
mod test;

pub use ant::{AntColor, AntDirection};
pub use instruction::{Instruction, TurnDir};
pub use simulator::{Outcome, Simulator};
pub use world::{Cell, World};
