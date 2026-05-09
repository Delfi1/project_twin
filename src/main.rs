//! Проект - Цифровой Двойник лука
//! Симуляция Онтогенеза растения; The influence of water and light on growth;
mod cell;

use bevy::prelude::*;
use cell::*;

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

fn init(mut commands: Commands, hexgrid: Res<HexGrid>) {
    info!("Initializing viewer...");

    commands.spawn((hexgrid.cell(), hexgrid.material(), hexgrid.mesh()));
}

#[derive(Default)]
pub struct Config {
    asset: Option<Handle<parser::Parser>>,
}

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

        commands.insert_resource(HexGrid::new(parser.clone(), &mut meshes, &mut materials));
        state.set(SimulationState::Viewer);
    }

    local.asset = Some(parser);
}

fn tick() {}

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
