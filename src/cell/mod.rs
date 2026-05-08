use bevy::prelude::*;
use std::sync::Arc;
pub mod parser;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Coord {
    q: isize,
    r: isize,
    s: isize,
}

pub enum Direction {}

pub struct Gen {}

pub struct Timer {}

#[derive(Component)]
pub struct Cell {
    gens: [Gen; 16],
    timers: [Timer; 4],
}

impl Cell {
    pub fn new(parser: &parser::Parser) -> Self {
        todo!("Todo parser loading new cell");
    }

    pub fn tick(&mut self, parser: Arc<parser::Parser>) {}
}
