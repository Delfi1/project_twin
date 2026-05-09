//! Проект - Цифровой Двойник лука
//! Симуляция Онтогенеза растения; The influence of water and light on growth;
mod camera;
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
    commands.spawn((Camera2d, camera::Controller));
    commands.insert_resource(ClearColor(Color::WHITE));

    // Скорость работы симуляции
    commands.insert_resource(Time::<Fixed>::from_seconds(0.25));
}

fn init(mut commands: Commands, mut hexgrid: ResMut<HexGrid>) {
    info!("Initializing viewer...");

    //hexgrid.add_cell(&mut commands, Coord::origin());
    let mut coords = Coord::ORIGIN;
    let mut direction = Direction::None;

    // Test generate hexes
    let mut i = 0;
    while i != 12 {
        coords = coords.neighbor(direction);
        if hexgrid.get(&coords).is_none() {
            hexgrid.insert(&mut commands, coords);
            i += 1;
        }

        direction = Direction::random();
    }
}

#[derive(Default)]
pub struct Config {
    asset: Option<Handle<parser::Config>>,
}

// Проверка загружен ли конфиг луковицы
fn config(
    mut commands: Commands,
    mut local: Local<Config>,
    asset_server: Res<AssetServer>,
    assets: Res<Assets<parser::Config>>,
    mut state: ResMut<NextState<SimulationState>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if local.asset.is_none() {
        local.asset = Some(asset_server.load("config.sim"));
    }

    let config = local.asset.take().unwrap();

    if let Some(config) = assets.get(&config) {
        info!("Config loaded...");

        let parent = commands.spawn((Origin, Transform::IDENTITY)).id();
        commands.insert_resource(HexGrid::new(
            parent,
            Arc::new(config.clone()),
            &mut meshes,
            &mut materials,
        ));
        state.set(SimulationState::World);
    }

    local.asset = Some(config);
}

fn tick(
    hexgrid: Res<HexGrid>,
    mut cells: Query<Mut<Cell>>,
    mut origin: Single<Mut<Transform>, With<Origin>>,
) {
    for mut _cell in cells.iter_mut() {
        //cell.tick(&hexgrid);
    }
}

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_state(SimulationState::Loading)
        .init_asset::<parser::Config>()
        .init_asset_loader::<parser::ConfigLoader>()
        .add_systems(PreStartup, setup)
        .add_systems(Update, config.run_if(in_state(SimulationState::Loading)))
        .add_systems(
            Update,
            camera::update.run_if(in_state(SimulationState::World)),
        )
        .add_systems(FixedUpdate, tick.run_if(in_state(SimulationState::World)))
        .add_systems(OnExit(SimulationState::Loading), init)
        .run();
}
