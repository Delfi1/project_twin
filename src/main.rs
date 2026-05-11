//! Проект - Цифровой Двойник лука
//! Симуляция Онтогенеза растения; The influence of water and light on growth;
mod grid;
mod hex;

use crate::grid::*;
use bevy::prelude::*;
use std::sync::Arc;

// Set current simulation type as Hex
pub type Simulation = hex::HexGrid;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum SimulationState {
    // Загрузка конфига генерации
    Loading,
    World,
    _Viewer,
}

#[derive(Component)]
pub struct Origin;

#[derive(Default)]
pub struct ConfigAsset {
    asset: Option<Handle<grid::Config>>,
}

// Проверка загружен ли конфиг луковицы
fn load_config(
    mut commands: Commands,
    mut local: Local<ConfigAsset>,
    asset_server: Res<AssetServer>,
    assets: Res<Assets<grid::Config>>,
    mut state: ResMut<NextState<SimulationState>>,
) {
    if local.asset.is_none() {
        local.asset = Some(asset_server.load("config.sim"));
    }

    let config = local.asset.take().unwrap();

    if let Some(config) = assets.get(&config).cloned() {
        info!("Config loaded...");

        let parent = commands.spawn((Origin, Transform::IDENTITY)).id();
        commands.insert_resource(Simulation::new(parent, Arc::new(config)));
        commands.init_resource::<<Simulation as Grid>::Materials>();
        // On config load grid creating
        state.set(SimulationState::World);
    }

    local.asset = Some(config);
}

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Time::<Fixed>::from_seconds(0.25))
        .insert_state(SimulationState::Loading)
        .init_asset::<grid::Config>()
        .init_asset_loader::<grid::ConfigLoader>()
        .add_systems(PreStartup, Simulation::on_setup)
        .add_systems(
            Update,
            load_config.run_if(in_state(SimulationState::Loading)),
        )
        .add_systems(
            Update,
            (
                Simulation::on_tick,
                <Simulation as Grid>::Controller::update,
            )
                .run_if(in_state(SimulationState::World)),
        )
        .add_systems(OnExit(SimulationState::Loading), Simulation::on_load)
        .run();
}
