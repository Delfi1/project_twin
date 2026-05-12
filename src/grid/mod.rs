pub mod config;

use bevy::{ecs::component::Mutable, prelude::*};
pub use config::*;
use std::{fmt::Debug, sync::Arc};

pub trait Direction: Sized + Clone + Copy + Default {
    // Get list of directions to a neighbors cells
    fn neighbors() -> &'static [Self];
    fn random() -> Self;
}

pub trait Coords: Clone + Copy + std::hash::Hash + Eq + PartialEq {
    type Dir: Direction;

    fn neighbor(&self, direction: &Self::Dir) -> Self;
}

pub trait Cell: Sized + Clone + Component<Mutability = Mutable> {
    // todo: simulation core operations with cells
    //fn tick();

    //fn gen(&self) -> bool;
    //fn can_division(&self) -> bool;
    fn new(_type: Arc<CellType>) -> Self;

    /// Клетка должна содержать ссылку на свой тип
    fn cell_type(&self) -> Arc<CellType>;

    /// Активен ли ген?
    fn is_active(&self, index: usize) -> bool;

    /// Работает ли таймер?
    fn is_running(&self, timer: usize) -> bool;
}

/// Основной абстрактный класс для работы с сеткой
pub trait Grid: Resource {
    type Cell: Cell;
    type Coords: Coords;
    type Materials: Resource + FromWorld;
    type Controller: Controller;
    type Origin: Component + Default;
    type Populate: Resource + FromWorld;

    fn new(parent: Entity, config: Arc<Config>) -> Self;

    /// Добавить клетку в сетку по координатам
    fn insert(
        &mut self,
        commands: &mut Commands,
        materials: &Self::Materials,
        coords: Self::Coords,
        cell: Self::Cell,
    );

    /// Получить клетку по координатам
    fn get(&self, coords: &Self::Coords) -> Option<&Entity>;

    /// Получить концентрацию морфогена на определенной позиции
    fn concentration(&self, coords: &Self::Coords) -> Option<&[bool; GENS]>;

    /// Количество соседей у данной клетки
    fn neighbors(&self, coords: &Self::Coords) -> u8 {
        let mut result = 0;
        for d in <Self::Coords as Coords>::Dir::neighbors() {
            result += self
                .get(&coords.neighbor(d))
                .and_then(|_| Some(1))
                .unwrap_or(0);
        }

        result
    }

    /// Свободные соседи
    fn free_neighbors(&self, coords: &Self::Coords) -> Vec<Self::Coords> {
        let mut result = Vec::with_capacity(16);
        for d in <Self::Coords as Coords>::Dir::neighbors() {
            let n = coords.neighbor(d);
            if self.get(&n).is_some() {
                continue;
            }

            result.push(n);
        }

        result
    }

    // Найстройка окружения Bevy
    fn on_setup(commands: Commands);

    /// Система которая подгружает сетку из конфигурации
    fn on_load(grid: ResMut<Self>, materials: Res<Self::Materials>, commands: Commands);

    /// Обновление морфогена
    fn prepare(grid: ResMut<Self>, cells: Query<Mut<Self::Cell>>);

    /// Обновление морфогена
    fn process(grid: Res<Self>, cells: Query<Mut<Self::Cell>>, populate: ResMut<Self::Populate>);

    fn spawn(
        grid: ResMut<Self>,
        commands: Commands,
        populate: ResMut<Self::Populate>,
        materials: Res<Self::Materials>,
    );

    // Система которая проверяет выбор объекта на сетке
    fn select(
        camera: Single<(Ref<Camera>, Ref<GlobalTransform>), With<Self::Controller>>,
        origin: Single<Ref<Transform>, With<Self::Origin>>,
        window: Single<Ref<Window>, With<bevy::window::PrimaryWindow>>,
        msb: Res<ButtonInput<MouseButton>>,
    );
}

/// Контроллер камеры для сетки
pub trait Controller: Component + Sized {
    const SPEED: f32 = 200.0;
    const ZOOMING: f32 = 0.1;
    const SCROLL: f32 = 0.8;

    // Обновление контроллера через Bevy
    fn update(
        time: Res<Time>,
        scroll: Local<f32>,
        kbd: Res<ButtonInput<KeyCode>>,
        scroll_msg: MessageReader<bevy::input::mouse::MouseWheel>,
        camera: Single<(Mut<Transform>, Mut<Projection>), With<Self>>,
    );
}
