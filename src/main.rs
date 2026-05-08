//! Project - Twin of real Onion
//! Simulation of plant ontogeny; The influence of water and light on growth;
mod cell;

use bevy::prelude::*;
use cell::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum SimulationState {
    Loading,
    World,
    Viewer,
}

fn setup() {}

// Проверка загружен ли конфиг луковицы
fn config(
    parser: Option<Res<parser::WorldParser>>,
    assets: Res<Assets<parser::Parser>>,
    mut state: ResMut<NextState<SimulationState>>,
) {
    let Some(parser) = parser else { return };

    if assets.get(&parser.0).is_some() {
        state.set(SimulationState::Viewer);
    }
}

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_state(SimulationState::Loading)
        .init_asset::<parser::Parser>()
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            config.run_if(in_state(SimulationState::Loading)),
        )
        .run();
}
