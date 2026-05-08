//! Project - Twin of real Onion
//! Simulation of plant ontogeny; The influence of water and light on growth;

use bevy::prelude::*;
mod cell;

fn setup() {}

fn tick() {}

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, tick)
        .run();
}
