pub mod config;

use bevy::{platform::collections::HashMap, prelude::*, window::*};
pub use config::*;
use rand::prelude::*;
use std::sync::Arc;

pub trait Direction: Sized + Clone + Copy + Default {
    // Get list of directions to a neighbors cells
    fn neighbors() -> &'static [Self];
    //fn random() -> Self;
}

pub trait Coords: Clone + Copy + std::hash::Hash + Eq + PartialEq {
    type Dir: Direction;
    type Iter: Iterator<Item = Self>;

    /// Конвертировать локальную координату в глобальную
    fn to_world(&self) -> Vec3;

    fn neighbor(&self, direction: &Self::Dir) -> Self;

    fn range(self, v: isize) -> Self::Iter;
}

pub trait Cell: Sized + Clone + Component<Mutability = bevy::ecs::component::Mutable> {
    fn new(_type: Arc<CellType>) -> Self;

    /// Клетка должна содержать ссылку на свой тип
    fn get_type(&self) -> Arc<CellType>;

    fn gens(&self) -> &[bool; GENS];
    fn gens_mut(&mut self) -> &mut [bool; GENS];
    fn timers(&self) -> &[u8; TIMERS];
    fn timers_mut(&mut self) -> &mut [u8; TIMERS];
    fn set_timer(&mut self, i: usize, v: u8);
    fn get_divide(&mut self) -> &mut bool;
}

/// Контроллер камеры для сетки
pub trait Controller: Component<Mutability = bevy::ecs::component::Mutable> + Sized {
    const SPEED: f32 = 200.0;
    const ZOOMING: f32 = 0.1;
    const SCROLL: f32 = 0.8;
    const SENSITIVITY: f32 = 0.002;

    // Обновление контроллера через Bevy
    fn update(
        time: Res<Time>,
        kbd: Res<ButtonInput<KeyCode>>,
        camera: Single<(Mut<Transform>, Mut<Self>)>,
        cursor: Single<Mut<CursorOptions>, With<PrimaryWindow>>,
        mouse: MessageReader<bevy::input::mouse::MouseMotion>,
    );

    fn scroll(
        mut scroll_msg: MessageReader<bevy::input::mouse::MouseWheel>,
        mut scroll: Local<f32>,
        projection: Single<Mut<Projection>, With<Self>>,
    ) {
        for m in scroll_msg.read() {
            *scroll -= m.y * Self::ZOOMING;
        }

        *scroll = scroll.clamp(-Self::SCROLL, Self::SCROLL);

        let zoom = 1.0 + *scroll;
        match *projection.into_inner() {
            Projection::Orthographic(ref mut orthographic) => {
                orthographic.scale = zoom;
            }
            _ => (),
        };
    }
}

#[derive(Resource)]
/// Клетки которые нужно будет создать в конце тика
pub struct SpawnQuery<C: Coords, T: Cell>(HashMap<C, T>);

unsafe impl<C: Coords, T: Cell> Send for SpawnQuery<C, T> {}
unsafe impl<C: Coords, T: Cell> Sync for SpawnQuery<C, T> {}

impl<C: Coords, T: Cell> Default for SpawnQuery<C, T> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<C: Coords, T: Cell> SpawnQuery<C, T> {
    pub fn insert(&mut self, coords: C, value: T) {
        self.0.insert(coords, value);
    }

    pub fn get_mut(&mut self) -> &mut HashMap<C, T> {
        &mut self.0
    }
}

#[derive(Resource)]
/// Концентрация морфогена в межклеточном веществе определенной клетки.
pub struct Concentrations<C: Coords>(HashMap<C, [u8; GENS]>);

unsafe impl<C: Coords> Send for Concentrations<C> {}
unsafe impl<C: Coords> Sync for Concentrations<C> {}

impl<C: Coords> Default for Concentrations<C> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<C: Coords> Concentrations<C> {
    pub fn insert(&mut self, coords: C, values: [u8; GENS]) {
        self.0.insert(coords, values);
    }

    pub fn get(&self, coords: &C) -> Option<&[u8; GENS]> {
        self.0.get(coords)
    }

    pub fn get_mut(&mut self, coords: &C) -> Option<&mut [u8; GENS]> {
        self.0.get_mut(coords)
    }
}

/// Основной абстрактный класс для работы с сеткой
pub trait Grid: Resource {
    type Cell: Cell;
    type Coords: Coords;
    type Origin: Component + FromWorld;
    type Controller: Controller;
    type Materials: Resource + FromWorld;

    fn new(parent: Entity, config: Arc<Config>) -> Self;

    fn get_tick(&self) -> u8;

    fn get_tick_mut(&mut self) -> &mut u8;

    fn tick(mut grid: ResMut<Self>) {
        *grid.get_tick_mut() = grid.get_tick().wrapping_add(1);
    }

    fn firstly(commands: &mut Commands) {
        commands.init_resource::<SpawnQuery<Self::Coords, Self::Cell>>();
        commands.init_resource::<Concentrations<Self::Coords>>();
    }

    /// Настройка окружения Bevy
    fn on_setup(commands: Commands);

    /// Система которая подгружает сетку из конфигурации
    fn on_load(
        grid: ResMut<Self>,
        concentrations: ResMut<Concentrations<Self::Coords>>,
        materials: Res<Self::Materials>,
        commands: Commands,
    );

    /// Обновление морфогена
    fn prepare(
        grid: ResMut<Self>,
        mut cells: Query<Mut<Self::Cell>>,
        mut concentrations: ResMut<Concentrations<Self::Coords>>,
    ) {
        let data = grid.get_data();
        let t = grid.get_tick();

        for (coords, entity) in data {
            let mut cell = cells.get_mut(*entity).unwrap();
            let _type = cell.get_type();

            for g in 0..GENS {
                if !cell.gens()[g] {
                    continue;
                }
                let Some(mgen) = _type.gens.get(&g) else {
                    continue;
                };

                for c in coords.range(mgen.range) {
                    if !data.contains_key(&c) {
                        continue;
                    };

                    let prev = concentrations.get_mut(&c).unwrap();
                    prev[g] = t;
                }
            }

            for timer in 0..TIMERS {
                let v = cell.timers()[timer];
                if v == 0 {
                    continue;
                }

                cell.set_timer(timer, v - 1);
            }
        }
    }

    /// Проверяем все условия, запускаем и останавливаем гены, меняем типы клетки и даже реплецируем их.
    /// Порядок проверок:
    /// 1) Дифференцировка (!пропустить остальные этапы, если верно)
    /// 2) Реплецирование клетки
    /// 3) Проверка на запуск генов/таймеров
    fn process(
        grid: Res<Self>,
        mut cells: Query<Mut<Self::Cell>>,
        mut spawn_q: ResMut<SpawnQuery<Self::Coords, Self::Cell>>,
        concentrations: Res<Concentrations<Self::Coords>>,
    ) {
        let mut rng = rand::rng();
        let config = grid.get_config();
        let t = grid.get_tick();

        for (coords, entity) in grid.get_data() {
            let mut cell = cells.get_mut(*entity).unwrap();
            let _type = cell.get_type();
            let n = grid.neighbors(coords);
            let c = concentrations.get(&coords).unwrap();
            let mut d = *cell.get_divide();

            // Этап 1:
            let mut skip = false;
            for (new, condition) in &_type.changes {
                if !condition.check(d, n, c, cell.timers(), &_type.name, t) {
                    continue;
                }

                let Some(new_type) = config.types.get(new) else {
                    warn!("Cell type is not found: {}", new);
                    continue;
                };

                // Дифференцировка типа клетки
                spawn_q.insert(*coords, Cell::new(new_type.clone()));
                skip = true;
                break;
            }

            if skip {
                continue;
            }

            // Этап 2
            *cell.get_divide() = match &_type.division {
                Some(condition) => condition.check(d, n, c, cell.timers(), &_type.name, t),
                None => false,
            };
            d = *cell.get_divide();

            if d {
                let neighbors = grid.empty_neighbors(coords);

                if neighbors.len() != 0 {
                    let nbr = neighbors.choose(&mut rng).unwrap();
                    spawn_q.insert(*nbr, Cell::new(_type.clone()));
                }
            }

            // Этап 3
            for (g, mgen) in _type.gens.iter() {
                cell.gens_mut()[*g] = mgen.condition.check(d, n, c, cell.timers(), &_type.name, t);
            }

            let mut timers = cell.timers().clone();
            for (tm, timer) in _type.timers.iter() {
                if !timer
                    .condition
                    .check(d, n, c, cell.timers(), &_type.name, t)
                {
                    continue;
                }

                timers[*tm] = timer.time;
            }

            *cell.timers_mut() = timers;
        }
    }

    fn spawn(
        mut grid: ResMut<Self>,
        mut commands: Commands,
        materials: Res<Self::Materials>,
        mut concentrations: ResMut<Concentrations<Self::Coords>>,
        mut spawn_q: ResMut<SpawnQuery<Self::Coords, Self::Cell>>,
    ) {
        for (coord, cell) in spawn_q.get_mut().drain() {
            if let Some(entity) =
                grid.insert(&mut commands, &mut concentrations, &materials, coord, cell)
            {
                commands.entity(entity).despawn();
            }
        }
    }

    /// Получить клетку по координатам
    fn get_data(&self) -> &HashMap<Self::Coords, Entity>;

    /// Получить клетку по координатам
    fn get(&self, coords: &Self::Coords) -> Option<&Entity>;

    /// Добавить клетку в сетку по координатам
    fn insert(
        &mut self,
        commands: &mut Commands,
        concentations: &mut Concentrations<Self::Coords>,
        materials: &Self::Materials,
        coords: Self::Coords,
        cell: Self::Cell,
    ) -> Option<Entity>;

    fn get_config(&self) -> Arc<Config>;

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
    fn empty_neighbors(&self, coords: &Self::Coords) -> Vec<Self::Coords> {
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

    /// Система которая проверяет выбор объекта на сетке
    fn select(
        commands: Commands,
        grid: ResMut<Self>,
        materials: Res<Self::Materials>,
        camera: Single<(Ref<Camera>, Ref<GlobalTransform>), With<Self::Controller>>,
        origin: Single<Ref<Transform>, With<Self::Origin>>,
        window: Single<Ref<Window>, With<bevy::window::PrimaryWindow>>,
        msb: Res<ButtonInput<MouseButton>>,
    );

    fn add_selection(
        &mut self,
        commands: &mut Commands,
        materials: &Self::Materials,
        coords: Self::Coords,
    );

    fn clear_selection(&mut self, commands: &mut Commands);
}
