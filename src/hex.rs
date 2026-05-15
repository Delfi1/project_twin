//! Гексагональная сетка реализованная с помощью базового трейта

use super::grid::*;
use bevy::{platform::collections::HashMap, prelude::*, window::*};
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

    //fn random() -> Self { Self::NEIGHBORS[rand::random_range(0..6)] }
}

pub struct RangeIter {
    start: HexCoords,
    end: HexCoords,

    // Текущие данные для перебора
    q: isize,
    r: isize,
    s: isize,
    finished: bool,
}

impl RangeIter {
    pub fn new(coords: HexCoords, range: isize) -> Self {
        let start = coords - range;
        let end = coords + range;
        Self {
            q: start.q,
            r: start.r,
            s: start.s,
            start,
            end,
            finished: false,
        }
    }
}

impl Iterator for RangeIter {
    type Item = HexCoords;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let s = self.s;
            let r = self.r;
            let q = self.q;

            self.s += 1;

            if self.s > self.end.s {
                self.s = self.start.s;
                self.r += 1;

                if self.r > self.end.r {
                    self.r = self.start.r;
                    self.q += 1;

                    if self.q > self.end.q {
                        self.finished = true;
                    }
                }
            }

            if q + r + s == 0 {
                return Some(HexCoords { q, r, s });
            }

            if self.finished {
                return None;
            }
        }
    }
}

#[derive(Clone, Debug, Copy, Hash, PartialEq, Eq)]
pub struct HexCoords {
    q: isize,
    r: isize,
    s: isize,
}

impl std::ops::Sub<isize> for HexCoords {
    type Output = Self;

    fn sub(mut self, rhs: isize) -> Self {
        self.q -= rhs;
        self.r -= rhs;
        self.s -= rhs;

        self
    }
}

impl std::ops::Add<isize> for HexCoords {
    type Output = Self;

    fn add(mut self, rhs: isize) -> Self {
        self.q += rhs;
        self.r += rhs;
        self.s += rhs;

        self
    }
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
    type Iter = RangeIter;

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

    fn range(self, v: isize) -> Self::Iter {
        RangeIter::new(self, v)
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
    fn get_type(&self) -> Arc<CellType> {
        self._type.clone()
    }

    #[inline]
    fn gens(&self) -> &[bool; GENS] {
        &self.gens
    }

    #[inline]
    fn gens_mut(&mut self) -> &mut [bool; GENS] {
        &mut self.gens
    }

    #[inline]
    fn timers(&self) -> &[u8; TIMERS] {
        &self.timers
    }

    #[inline]
    fn timers_mut(&mut self) -> &mut [u8; TIMERS] {
        &mut self.timers
    }

    #[inline]
    fn set_timer(&mut self, i: usize, v: u8) {
        self.timers[i] = v;
    }

    #[inline]
    fn get_divide(&mut self) -> &mut bool {
        &mut self.d
    }
}

#[derive(Component)]
pub struct HexController;

impl Controller for HexController {
    // Простой контроллер 2d камеры
    fn update(
        time: Res<Time>,
        kbd: Res<ButtonInput<KeyCode>>,
        camera: Single<(Mut<Transform>, Mut<Self>)>,
        _cursor: Single<Mut<CursorOptions>, With<PrimaryWindow>>,
        _mouse: MessageReader<bevy::input::mouse::MouseMotion>,
    ) {
        let (mut camera, _) = camera.into_inner();

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

        if velocity != Vec3::ZERO {
            camera.translation += velocity.normalize() * Self::SPEED * time.delta_secs();
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
pub struct HexGrid {
    pub config: Arc<Config>,
    parent: Entity,

    selected: Option<(HexCoords, Entity)>,
    data: HashMap<HexCoords, Entity>,
    current_tick: u8,
}

impl Grid for HexGrid {
    type Cell = HexCell;
    type Coords = HexCoords;
    type Controller = HexController;
    type Materials = HexMaterials;
    type Origin = HexOrigin;

    fn new(parent: Entity, config: Arc<Config>) -> Self {
        let data = HashMap::new();

        Self {
            config,
            parent,
            selected: None,
            data,
            current_tick: 128,
        }
    }

    fn get_tick(&self) -> u8 {
        self.current_tick
    }

    fn get_tick_mut(&mut self) -> &mut u8 {
        &mut self.current_tick
    }

    /// Двумерный мир с серым фоном
    fn on_setup(mut commands: Commands) {
        commands.spawn((Camera2d, HexController));
        commands.insert_resource(ClearColor(Color::srgb_u8(43, 43, 43)));
    }

    fn on_load(
        mut grid: ResMut<Self>,
        mut concentrations: ResMut<Concentrations<HexCoords>>,
        materials: Res<Self::Materials>,
        mut commands: Commands,
    ) {
        info!("Initializing grid...");

        let coords = Self::Coords::ORIGIN;
        let _type = grid.config.types.get(&grid.config.default).unwrap().clone();
        let mut cell = HexCell::new(_type);
        cell.gens[0] = true;

        grid.insert(&mut commands, &mut concentrations, &materials, coords, cell);
    }

    fn get_data(&self) -> &HashMap<Self::Coords, Entity> {
        &self.data
    }

    fn get(&self, coords: &Self::Coords) -> Option<&Entity> {
        self.data.get(coords)
    }

    fn insert(
        &mut self,
        commands: &mut Commands,
        concentrations: &mut Concentrations<HexCoords>,
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

        concentrations.insert(coords, [self.current_tick - 2; GENS]);
        self.data.insert(coords, entity)
    }

    fn get_config(&self) -> Arc<Config> {
        self.config.clone()
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
}
