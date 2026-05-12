//! Гексагональная сетка реализованная с помощью базового трейта

use super::grid::*;
use bevy::{platform::collections::HashMap, prelude::*};
use rand::seq::IndexedRandom;
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

#[derive(Clone, Debug, Copy, Hash, PartialEq, Eq)]
pub struct HexCoords {
    q: isize,
    r: isize,
    s: isize,
}

impl HexCoords {
    pub const ORIGIN: Self = Self::new(0, 0);

    pub const fn round(q0: f32, r0: f32, s0: f32) -> Self {
        let mut q = q0.round();
        let mut r = r0.round();
        let s = s0.round();

        let q_diff = (q - q0).abs();
        let r_diff = (r - r0).abs();
        let s_diff = (s - s0).abs();

        if q_diff > r_diff && q_diff > s_diff {
            q = -r - s;
        } else if r_diff > s_diff {
            r = -q - s;
        }

        Self::new(q as isize, r as isize)
    }

    pub const fn new(q: isize, r: isize) -> Self {
        Self { q, r, s: -q - r }
    }
}

impl Coords for HexCoords {
    type Dir = HexDirection;

    fn to_world(&self) -> Vec3 {
        Vec3::new(
            SQRT3 * self.q as f32 + SQRT3 / 2.0 * self.r as f32,
            3. / 2. * self.r as f32,
            0.0,
        )
        .mul(SIZE)
    }

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
pub const SQRT3: f32 = 1.73205;
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
    pub selected_mesh: Mesh2d,
    pub selected_material: MeshMaterial2d<ColorMaterial>,
    pub materials: HashMap<String, MeshMaterial2d<ColorMaterial>>,
}

impl FromWorld for HexMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let mesh = Mesh2d(meshes.add(RegularPolygon::new(SIZE, 6).to_ring(THINKNESS)));
        let selected_mesh = Mesh2d(meshes.add(RegularPolygon::new(SIZE, 6)));

        let grid = world.get_resource::<HexGrid>().unwrap();
        let config = grid.config.clone();

        let mut color_materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        let mut materials = HashMap::with_capacity(config.types.capacity());
        for (name, t) in config.types.iter() {
            let material = color_materials.add(Color::Srgba(t.color));
            materials.insert(name.clone(), MeshMaterial2d(material));
        }

        let selected_material = MeshMaterial2d(color_materials.add(Color::srgb_u8(0, 120, 215)));

        Self {
            mesh,
            selected_mesh,
            selected_material,
            materials,
        }
    }
}

#[derive(Default, Component)]
pub struct HexOrigin;

#[derive(Resource)]
pub struct HexPopulate {
    to_spawn: HashMap<HexCoords, HexCell>,
}

impl Default for HexPopulate {
    fn default() -> Self {
        Self {
            to_spawn: HashMap::with_capacity(2048),
        }
    }
}

#[derive(Resource)]
pub struct HexGrid {
    pub config: Arc<Config>,
    running: bool,
    // Родительский элемент от которого уже отрисовываются все клетки
    parent: Entity,

    selected: Option<(HexCoords, Entity)>,
    data: HashMap<HexCoords, Entity>,
    // Концентрация морфогена в межклеточном веществе определенной клетки.
    // Постепенно уменьшается до нуля, если не восполнять.
    concentration: HashMap<HexCoords, [bool; GENS]>,
}

impl Grid for HexGrid {
    type Cell = HexCell;
    type Coords = HexCoords;
    type Controller = HexController;
    type Materials = HexMaterials;
    type Origin = HexOrigin;
    type Populate = HexPopulate;

    fn new(parent: Entity, config: Arc<Config>) -> Self {
        let data = HashMap::new();
        let concentration = HashMap::new();

        Self {
            config,
            parent,
            running: true,
            selected: None,
            data,
            concentration,
        }
    }

    fn stop(mut grid: ResMut<Self>, kbd: Res<ButtonInput<KeyCode>>) {
        if kbd.just_pressed(KeyCode::Space) {
            grid.running = !grid.running;
        }
    }

    fn is_running(res: Option<Res<Self>>) -> bool {
        let Some(grid) = res else {
            return false;
        };

        grid.running
    }

    fn insert(
        &mut self,
        commands: &mut Commands,
        materials: &Self::Materials,
        coords: Self::Coords,
        cell: Self::Cell,
    ) -> Option<Entity> {
        let material = materials.materials.get(&cell._type.name).unwrap();

        let pos = coords.to_world();

        let entity = commands
            .spawn((
                cell,
                materials.mesh.clone(),
                material.clone(),
                Transform::from_translation(pos).with_scale(Vec3::splat(0.9)),
            ))
            .id();
        commands.entity(self.parent).add_child(entity);

        self.concentration.insert(coords, [false; GENS]);
        self.data.insert(coords, entity)
    }

    fn get(&self, coords: &Self::Coords) -> Option<&Entity> {
        self.data.get(coords)
    }

    fn concentration(&self, coords: &Self::Coords) -> Option<&[bool; GENS]> {
        self.concentration.get(coords)
    }

    /// Двумерный мир с серым фоном
    fn on_setup(mut commands: Commands) {
        commands.spawn((Camera2d, HexController));
        commands.insert_resource(ClearColor(Color::srgb_u8(43, 43, 43)));
    }

    fn on_load(mut grid: ResMut<Self>, materials: Res<Self::Materials>, mut commands: Commands) {
        info!("Initializing grid...");

        let coords = Self::Coords::ORIGIN;
        let _type = grid.config.types.get(&grid.config.default).unwrap().clone();
        let mut cell = HexCell::new(_type);
        cell.gens[0] = true;
        grid.insert(&mut commands, &materials, coords, cell);
    }

    fn select(
        mut commands: Commands,
        mut grid: ResMut<Self>,
        materials: Res<Self::Materials>,
        camera: Single<(Ref<Camera>, Ref<GlobalTransform>), With<Self::Controller>>,
        origin: Single<Ref<Transform>, With<Self::Origin>>,
        window: Single<Ref<Window>, With<bevy::window::PrimaryWindow>>,
        msb: Res<ButtonInput<MouseButton>>,
    ) {
        let (camera, camera_transform) = camera.into_inner();

        if msb.just_pressed(MouseButton::Right) {
            grid.clear_selection(&mut commands);
            return;
        }

        if !msb.just_pressed(MouseButton::Left) {
            return;
        }

        let cursor = window.cursor_position().unwrap();
        let viewport = camera
            .viewport_to_world_2d(&camera_transform, cursor)
            .unwrap()
            - origin.translation.xy();

        let x = (viewport.x) / SIZE;
        let y = (viewport.y) / SIZE;
        let q0 = x * SQRT3 / 3. - y / 3.;
        let r0 = y * 2. / 3.;
        let s0 = -q0 - r0;

        let coords = HexCoords::round(q0, r0, s0);
        grid.add_selection(&mut commands, &materials, coords);
    }

    fn add_selection(
        &mut self,
        commands: &mut Commands,
        materials: &Self::Materials,
        coords: Self::Coords,
    ) {
        self.clear_selection(commands);
        if !self.data.contains_key(&coords) {
            return;
        }

        let mesh = materials.selected_mesh.clone();
        let material = materials.selected_material.clone();
        let pos = coords.to_world().with_z(-1.0);

        let entity = commands
            .spawn((mesh, material, Transform::from_translation(pos)))
            .id();
        commands.entity(self.parent).add_child(entity);

        self.selected = Some((coords, entity));
    }

    fn clear_selection(&mut self, commands: &mut Commands) {
        if let Some((_, entity)) = self.selected {
            commands.entity(entity).despawn();
        }

        self.selected = None;
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

                            let prev = concentration.get_mut(&c).unwrap();
                            prev[g] = true;
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
    fn process(
        grid: Res<Self>,
        mut cells: Query<Mut<Self::Cell>>,
        populate: ResMut<Self::Populate>,
    ) {
        let Self::Populate { to_spawn } = &mut populate.into_inner();
        let mut rng = rand::rng();

        for (coords, entity) in grid.data.iter() {
            let mut cell = cells.get_mut(*entity).unwrap();
            let _type = cell.cell_type();
            let n = grid.neighbors(coords);
            let c = grid.concentration(&coords).unwrap();

            // Этап 1:
            let mut skip = false;
            for (new, condition) in &_type.changes {
                if !condition.check(cell.d, n, c, &cell.timers, &_type.name) {
                    continue;
                }

                let Some(new_type) = grid.config.types.get(new) else {
                    warn!("Cell type is not found: {}", new);
                    continue;
                };

                // Дифференцировка типа клетки
                to_spawn.insert(*coords, Cell::new(new_type.clone()));
                skip = true;
                break;
            }

            if skip {
                continue;
            }

            // Этап 2
            cell.d = match &_type.division {
                Some(condition) => condition.check(cell.d, n, c, &cell.timers, &_type.name),
                None => false,
            };

            if cell.d {
                let neighbors = grid.free_neighbors(coords);

                if neighbors.len() != 0 {
                    let nbr = neighbors.choose(&mut rng).unwrap();
                    to_spawn.insert(*nbr, Cell::new(_type.clone()));
                }
            }

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

    fn spawn(
        mut grid: ResMut<Self>,
        mut commands: Commands,
        mut populate: ResMut<Self::Populate>,
        materials: Res<Self::Materials>,
    ) {
        for (coord, cell) in populate.to_spawn.drain() {
            if let Some(entity) = grid.insert(&mut commands, &materials, coord, cell) {
                commands.entity(entity).despawn();
            }
        }
    }
}
