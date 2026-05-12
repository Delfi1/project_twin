//! Гексагональная сетка реализованная с помощью базового трейта

use super::grid::*;
use bevy::{platform::collections::HashMap, prelude::*};
use std::ops::Mul;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, Default)]
pub enum HexDirection {
    #[default]
    None,
    East,
    West,
    Northeast,
    Southwest,
    Northwest,
    Southeast,
}

impl HexDirection {
    pub const NEIGHBORS: [Self; 6] = [
        Self::East,
        Self::Northeast,
        Self::Southeast,
        Self::West,
        Self::Southwest,
        Self::Northwest,
    ];
}

impl Direction for HexDirection {
    fn neighbors() -> &'static [Self] {
        &Self::NEIGHBORS
    }

    fn random() -> Self {
        Self::NEIGHBORS[rand::random_range(0..6)]
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct HexCoords {
    q: isize,
    r: isize,
    s: isize,
}

impl HexCoords {
    pub const ORIGIN: Self = Self::new(0, 0);

    pub const fn new(q: isize, r: isize) -> Self {
        Self { q, r, s: -q - r }
    }
}

impl Coords for HexCoords {
    type Dir = HexDirection;

    fn neighbor(&self, dir: &Self::Dir) -> Self {
        match dir {
            Self::Dir::None => *self,
            Self::Dir::East => Self::new(self.q + 1, self.r),
            Self::Dir::West => Self::new(self.q - 1, self.r),
            Self::Dir::Northwest => Self::new(self.q, self.r - 1),
            Self::Dir::Southeast => Self::new(self.q, self.r + 1),
            Self::Dir::Northeast => Self::new(self.q + 1, self.r - 1),
            Self::Dir::Southwest => Self::new(self.q - 1, self.r + 1),
        }
    }
}

pub const SIZE: f32 = 16.0;
// Внутренний радиус гексагона, корень из 3
pub const INNER_RADIUS: f32 = 1.73205;
pub const THINKNESS: f32 = 4.0;

#[derive(Component, Clone)]
pub struct HexCell {
    _type: Arc<CellType>,
    pub gens: [bool; GENS],
    pub timers: [u8; TIMERS],
    // Могла ли клетка делится в предыдущий тик
    d: bool,
}

impl Cell for HexCell {
    // M0 is always active on init
    fn new(_type: Arc<CellType>) -> Self {
        let timers = [0; TIMERS];
        let gens = [false; GENS];

        Self {
            _type,
            gens,
            timers,
            d: false,
        }
    }

    #[inline]
    fn cell_type(&self) -> Arc<CellType> {
        self._type.clone()
    }

    #[inline]
    fn is_active(&self, index: usize) -> bool {
        self.gens[index]
    }

    #[inline]
    fn is_running(&self, timer: usize) -> bool {
        self.timers[timer] != 0
    }
}

#[derive(Component)]
pub struct HexController;

impl Controller for HexController {
    // Простой контроллер 2d камеры
    fn update(
        time: Res<Time>,
        mut scroll: Local<f32>,
        kbd: Res<ButtonInput<KeyCode>>,
        mut scroll_msg: MessageReader<bevy::input::mouse::MouseWheel>,
        camera: Single<(Mut<Transform>, Mut<Projection>), With<Self>>,
    ) {
        let (mut transform, projection) = camera.into_inner();

        for m in scroll_msg.read() {
            *scroll -= m.y * Self::ZOOMING;
        }
        *scroll = scroll.clamp(-Self::SCROLL, Self::SCROLL);

        let mut velocity = Vec3::ZERO;
        if kbd.pressed(KeyCode::KeyW) {
            velocity.y += 1.0;
        }
        if kbd.pressed(KeyCode::KeyA) {
            velocity.x -= 1.0;
        }
        if kbd.pressed(KeyCode::KeyS) {
            velocity.y -= 1.0;
        }
        if kbd.pressed(KeyCode::KeyD) {
            velocity.x += 1.0;
        }

        let zoom = 1.0 + *scroll;
        match *projection.into_inner() {
            Projection::Orthographic(ref mut orthographic) => {
                orthographic.scale = zoom;
            }
            _ => (),
        };

        if velocity != Vec3::ZERO {
            transform.translation += velocity.normalize() * Self::SPEED * time.delta_secs() * zoom;
        }
    }
}

#[derive(Resource)]
pub struct HexMaterials {
    pub mesh: Mesh2d,
    pub materials: HashMap<String, MeshMaterial2d<ColorMaterial>>,
}

impl FromWorld for HexMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let mesh = Mesh2d(meshes.add(RegularPolygon::new(SIZE, 6).to_ring(THINKNESS)));

        let grid = world.get_resource::<HexGrid>().unwrap();
        let config = grid.config.clone();

        let mut color_materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        let mut materials = HashMap::with_capacity(config.types.capacity());
        for (name, t) in config.types.iter() {
            let material = color_materials.add(Color::Srgba(t.color));
            materials.insert(name.clone(), MeshMaterial2d(material));
        }

        Self { mesh, materials }
    }
}

#[derive(Default, Component)]
pub struct HexOrigin;

#[derive(Resource)]
pub struct HexGrid {
    pub config: Arc<Config>,
    // Родительский элемент от которого уже отрисовываются все клетки
    parent: Entity,

    data: HashMap<HexCoords, Entity>,
    // Концентрация морфогена в межклеточном веществе определенной клетки.
    // Постепенно уменьшается до нуля, если не восполнять.
    concentration: HashMap<HexCoords, [u8; GENS]>,
}

impl Grid for HexGrid {
    type Cell = HexCell;
    type Coords = HexCoords;
    type Controller = HexController;
    type Materials = HexMaterials;
    type Origin = HexOrigin;

    fn new(parent: Entity, config: Arc<Config>) -> Self {
        let data = HashMap::new();
        let concentration = HashMap::new();

        Self {
            config,
            parent,
            data,
            concentration,
        }
    }

    fn insert(
        &mut self,
        commands: &mut Commands,
        materials: &Self::Materials,
        coords: Self::Coords,
        cell_type: Arc<CellType>,
    ) {
        let material = materials.materials.get(&self.config.default).unwrap();

        let pos = Vec3::new(
            INNER_RADIUS * coords.q as f32 + INNER_RADIUS / 2.0 * coords.r as f32,
            3. / 2. * coords.r as f32,
            0.0,
        )
        .mul(SIZE);

        let cell = HexCell::new(cell_type);
        let entity = commands
            .spawn((
                cell,
                materials.mesh.clone(),
                material.clone(),
                Transform::from_translation(pos).with_scale(Vec3::splat(0.9)),
            ))
            .id();
        commands.entity(self.parent).add_child(entity);

        self.data.insert(coords, entity);
        self.concentration.insert(coords, [0; GENS]);
    }

    fn get(&self, coords: &Self::Coords) -> Option<&Entity> {
        self.data.get(coords)
    }

    fn concentration(&self, coords: &Self::Coords) -> Option<&[u8; GENS]> {
        self.concentration.get(coords)
    }

    /// Двумерный мир с серым фоном
    fn on_setup(mut commands: Commands) {
        commands.spawn((Camera2d, HexController));
        commands.insert_resource(ClearColor(Color::srgb_u8(43, 43, 43)));
    }

    fn on_load(mut grid: ResMut<Self>, materials: Res<Self::Materials>, mut commands: Commands) {
        info!("Initializing grid...");

        let mut coords = Self::Coords::ORIGIN;
        let mut direction = HexDirection::None;

        // Тестовая генерация хексов
        let mut i = 0;
        while i != 12 {
            coords = coords.neighbor(&direction);
            let _type = grid.config.types.get(&grid.config.default).unwrap().clone();
            if grid.get(&coords).is_none() {
                grid.insert(&mut commands, &materials, coords, _type);
                i += 1;
            }

            direction = Direction::random();
        }
    }

    fn select(camera: Single<(Mut<Transform>, Mut<Projection>), With<Self::Controller>>) {
        todo!()
    }

    /// Порядок подготовки:
    /// 1) Выработка гена в определенном количестве в межклеточном веществе в радиусе.
    /// 2) Проверка на активацию/деактивацию генов
    fn prepare(grid: ResMut<Self>, mut cells: Query<Mut<Self::Cell>>) {
        let HexGrid {
            data,
            concentration,
            ..
        } = &mut grid.into_inner();

        // Производим морфогены
        for (coords, entity) in data.iter() {
            let mut cell = cells.get_mut(*entity).unwrap();
            let _type = cell.cell_type();

            for g in 0..GENS {
                let mut prev = concentration.get_mut(coords).unwrap();
                if prev[g] != 0 {
                    prev[g] -= 1;
                }

                if !cell.gens[g] {
                    continue;
                }
                let Some(mgen) = _type.gens.get(&g) else {
                    continue;
                };

                for q in coords.q - mgen.range..=coords.q + mgen.range {
                    for r in coords.r - mgen.range..=coords.r + mgen.range {
                        for s in coords.s - mgen.range..=coords.s + mgen.range {
                            if q + r + s != 0 {
                                continue;
                            }

                            let c = HexCoords { q, r, s };
                            if !data.contains_key(&c) {
                                continue;
                            };

                            // TODO: Сделать функцию распространения морфогена в зависомости от расстояния
                            prev = concentration.get_mut(&c).unwrap();
                            prev[g] = 4;
                        }
                    }
                }
            }

            for t in 0..TIMERS {
                if cell.timers[t] == 0 {
                    continue;
                }

                cell.timers[t] -= 1;
            }
        }
    }

    /// Проверяем все условия, запускаем и останавливаем гены, меняем типы клетки и даже реплецируем их.
    /// Порядок проверок:
    /// 1) Дифференцировка (!пропустить остальные этапы, если верно)
    /// 2) Реплецирование клетки
    /// 3) Проверка на запуск генов/таймеров
    fn process(grid: ResMut<Self>, mut cells: Query<Mut<Self::Cell>>) {
        for (coords, entity) in grid.data.iter() {
            let mut cell = cells.get_mut(*entity).unwrap();
            let _type = cell.cell_type();
            let n = grid.neighbors(coords);
            let c = grid.concentration(&coords).unwrap();

            // TODO: Этап 1:
            for (new, condition) in &_type.changes {
                if !condition.check(cell.d, n, c, &cell.timers, &_type.name) {
                    continue;
                }

                let Some(_type) = grid.config.types.get(new) else {
                    warn!("Cell type is not found: {}", new);
                    continue;
                };

                *cell = Cell::new(_type.clone());
                return;
            }

            // TODO: Этап 2
            cell.d = match &_type.division {
                Some(condition) => condition.check(cell.d, n, c, &cell.timers, &_type.name),
                None => false,
            };

            // Этап 3
            for (g, mgen) in _type.gens.iter() {
                cell.gens[*g] = mgen
                    .condition
                    .check(cell.d, n, c, &cell.timers, &_type.name);
            }

            let mut timers = cell.timers.clone();
            for (t, timer) in _type.timers.iter() {
                if !timer
                    .condition
                    .check(cell.d, n, c, &cell.timers, &_type.name)
                {
                    continue;
                }

                timers[*t] = timer.time;
            }
            cell.timers = timers;
        }
    }
}
