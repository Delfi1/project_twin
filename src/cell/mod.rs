pub mod parser;

use bevy::prelude::*;
use std::sync::Arc;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Coord {
    q: isize,
    r: isize,
    s: isize,
}

pub enum Direction {}

pub struct Timer {
    index: usize,
    current: u8,
}

impl Timer {
    pub fn tick(&mut self) -> bool {
        self.current -= 1;
        self.current == 0
    }
}

#[derive(Component)]
pub struct Cell {
    gens: Vec<parser::Value>,
    timers: Vec<Timer>,
}

impl Cell {
    pub fn new(_parser: &parser::Parser) -> Self {
        todo!()
    }

    pub fn tick(&mut self, _parser: Arc<parser::Parser>) {
        for mut _t in self.timers.drain(..) {
            todo!();
        }
    }
}
