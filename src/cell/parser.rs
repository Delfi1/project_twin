//! Genetic information gen.sim files parser
// File signature:
// --------------------------------------
// <NAME>: ACTIVATOR | DEACTIVATOR
// M0(Radius): M0
// T0[TICKS]: M0 M1 M2 | M3 M4
// --------------------------------------
// Index is selected by first number of gen
//
// Gen types:
// M(index) <- Morphogen
// T(index) <- Timer. Ticks to wait after activation

use bevy::prelude::*;
use std::sync::Arc;

/// Parser reads this Value from file
#[derive(Debug, Clone, Copy)]
pub enum Value {
    Gen {},
    Timer {},
}

pub const GENS: usize = 16;
pub const TIMERS: usize = 4;

#[derive(Asset, TypePath, Debug)]
pub struct Parser {
    pub gens: [Value; GENS],
    pub timers: [Value; TIMERS],
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            gens: [Value::Gen {}; GENS],
            timers: [Value::Timer {}; TIMERS],
        }
    }
}

impl Parser {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
}
