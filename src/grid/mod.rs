pub mod config;

use bevy::prelude::*;
pub use config::*;
use std::sync::Arc;

pub trait Direction: Sized + Clone + Copy + Default {
    // Get list of directions to a neighbors cells
    fn neighbors() -> &'static [Self];
    fn random() -> Self;
}

pub trait Coords: Clone + Copy + std::hash::Hash + Eq + PartialEq {
    type Dir: Direction;

    fn new(x: isize, y: isize, z: isize) -> Self;
    fn neighbor(&self, direction: Self::Dir) -> Self;
}

pub trait Cell: Component + Default {
    // todo: simulation core operations with cells
    //fn tick();
}

/// Основной абстрактный класс для работы с сеткой
pub trait Grid: Resource {
    type Cell: Cell;
    type Coords: Coords;
    type Controller: Controller;

    fn new(
        parent: Entity,
        config: Arc<Config>,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Self;

    /// Добавить клетку в сетку по координатам
    fn insert(&mut self, commands: &mut Commands, coords: Self::Coords, cell: Self::Cell);

    /// Получить клетку по координатам
    fn get(&self, coords: &Self::Coords) -> Option<&Entity>;

    /// Количество соседей у данной клетки
    fn neighbors(&self, coords: Self::Coords) -> u8;

    // Найстройка окружения Bevy
    fn on_setup(commands: Commands);

    /// Система которая подгружает сетку из конфигурации
    fn on_load(grid: ResMut<Self>, commands: Commands);

    /// Обновление сетки через Bevy
    fn on_tick(grid: Res<Self>);
}

/// Контроллер камеры для сетки
pub trait Controller {
    const SPEED: f32 = 200.0;
    const ZOOMING: f32 = 0.1;
    const SCROLL: f32 = 0.8;

    // Обновление контроллера через Bevy
    fn update(
        time: Res<Time>,
        scroll: Local<f32>,
        kbd: Res<ButtonInput<KeyCode>>,
        scroll_msg: MessageReader<bevy::input::mouse::MouseWheel>,
        camera: Single<(Mut<Transform>, Mut<Projection>), With<Camera>>,
    );
}
