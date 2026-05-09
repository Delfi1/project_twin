//! Проект - Цифровой Двойник лука
//! Симуляция Онтогенеза растения; The influence of water and light on growth;
mod cell;

use bevy::prelude::*;
use cell::*;
use std::sync::Arc;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum SimulationState {
    // Загрузка конфига генерации
    Loading,
    World,
    Viewer,
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, Msaa::Sample4));
    commands.insert_resource(ClearColor(Color::WHITE));
}

fn init(mut commands: Commands, mut hexgrid: ResMut<HexGrid>) {
    info!("Initializing viewer...");

    //hexgrid.add_cell(&mut commands, Coord::origin());
    hexgrid.add_cell(&mut commands, Coord::new(5, 0));
}

#[derive(Default)]
pub struct Config {
    asset: Option<Handle<parser::Parser>>,
}

#[derive(Component)]
// Точка отчёта симуляции, если сдвинуть её, сдвинется вся сетка
pub struct Origin;

// Проверка загружен ли конфиг луковицы
fn config(
    mut commands: Commands,
    mut local: Local<Config>,
    asset_server: Res<AssetServer>,
    assets: Res<Assets<parser::Parser>>,
    mut state: ResMut<NextState<SimulationState>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if local.asset.is_none() {
        local.asset = Some(asset_server.load("config.sim"));
    }

    let parser = local.asset.take().unwrap();

    if let Some(parser) = assets.get(&parser) {
        info!("Config loaded...");

        let parent = commands.spawn((Origin, Transform::IDENTITY)).id();
        commands.insert_resource(HexGrid::new(
            parent,
            Arc::new(parser.clone()),
            &mut meshes,
            &mut materials,
        ));
        state.set(SimulationState::Viewer);
    }

    local.asset = Some(parser);
}

fn tick(
    hexgrid: Res<HexGrid>,
    mut cells: Query<Mut<Cell>>,
    mut origin: Single<Mut<Transform>, With<Origin>>,
) {
    for mut _cell in cells.iter_mut() {
        //cell.tick(&hexgrid);
    }

    origin.rotate_z(3.14 / 128.);
}

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_state(SimulationState::Loading)
        .init_asset::<parser::Parser>()
        .init_asset_loader::<parser::ParserLoader>()
        .add_systems(Startup, setup)
        .add_systems(Update, config.run_if(in_state(SimulationState::Loading)))
        .add_systems(Update, tick.run_if(in_state(SimulationState::Viewer)))
        .add_systems(OnExit(SimulationState::Loading), init)
        .run();
}
