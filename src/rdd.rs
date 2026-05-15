use bevy::ecs::component::Component;
use std::ops::Mul;

use super::grid::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::{platform::collections::HashMap, prelude::*, window::*};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, Default)]
pub enum RddDirection {
    #[default]
    None,

    East,
    West,
    North,
    South,

    Up,
    UpNorthEast,
    UpNorth,
    UpEast,

    Down,
    DownSouthWest,
    DownWest,
    DownSouth,
}

impl RddDirection {
    pub const NEIGHBORS: [Self; 12] = [
        Self::East,
        Self::West,
        Self::North,
        Self::South,
        Self::Up,
        Self::UpNorthEast,
        Self::UpNorth,
        Self::UpEast,
        Self::Down,
        Self::DownSouthWest,
        Self::DownWest,
        Self::DownSouth,
    ];
}

impl Direction for RddDirection {
    fn neighbors() -> &'static [Self] {
        &Self::NEIGHBORS
    }
}

pub const SIZE: f32 = 1.0;
pub const SQRT2: f32 = 1.41421;

pub struct RangeIter {
    start: RddCoords,
    end: RddCoords,

    // Текущие данные для перебора
    x: isize,
    y: isize,
    z: isize,
    finished: bool,
}

impl RangeIter {
    pub fn new(coords: RddCoords, range: isize) -> Self {
        let start = coords - range;
        let end = coords + range;
        Self {
            x: start.z,
            y: start.y,
            z: start.z,
            start,
            end,
            finished: false,
        }
    }
}

impl Iterator for RangeIter {
    type Item = RddCoords;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        loop {
            let x = self.x;
            let y = self.y;
            let z = self.z;

            self.x += 1;

            if self.x > self.end.x {
                self.x = self.start.x;
                self.y += 1;

                if self.y > self.end.y {
                    self.y = self.start.y;
                    self.z += 1;

                    if self.z > self.end.z {
                        self.finished = true;
                    }
                }
            }

            return Some(RddCoords { x, y, z });
        }
    }
}

#[derive(Clone, Debug, Copy, Hash, PartialEq, Eq)]
pub struct RddCoords {
    x: isize,
    y: isize,
    z: isize,
}

impl std::ops::Sub<isize> for RddCoords {
    type Output = Self;

    fn sub(mut self, rhs: isize) -> Self {
        self.x -= rhs;
        self.y -= rhs;
        self.z -= rhs;

        self
    }
}

impl std::ops::Add<isize> for RddCoords {
    type Output = Self;

    fn add(mut self, rhs: isize) -> Self {
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;

        self
    }
}

impl RddCoords {
    pub const ORIGIN: Self = Self::new(0, 0, 0);

    pub const fn new(x: isize, y: isize, z: isize) -> Self {
        Self { x, y, z }
    }

    pub fn to_local(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3::new((2. * y - z) / SQRT2, (-2. * x + z) / SQRT2, z).mul(SIZE)
    }
}

impl Coords for RddCoords {
    type Dir = RddDirection;
    type Iter = RangeIter;

    fn to_world(&self) -> Vec3 {
        let x = self.x as f32;
        let y = self.y as f32;
        let z = self.z as f32;

        Vec3::new((2. * y - z) / SQRT2, (-2. * x + z) / SQRT2, z).mul(SIZE)
    }

    fn neighbor(&self, dir: &Self::Dir) -> Self {
        match dir {
            Self::Dir::None => *self,
            Self::Dir::East => Self::new(self.x + 1, self.y, self.z),
            Self::Dir::West => Self::new(self.x - 1, self.y, self.z),
            Self::Dir::North => Self::new(self.x, self.y, self.z + 1),
            Self::Dir::South => Self::new(self.x, self.y, self.z - 1),

            Self::Dir::Up => Self::new(self.x, self.y + 1, self.z),
            Self::Dir::UpNorthEast => Self::new(self.x + 1, self.y + 1, self.z + 1),
            Self::Dir::UpNorth => Self::new(self.x, self.y + 1, self.z + 1),
            Self::Dir::UpEast => Self::new(self.x + 1, self.y + 1, self.z),

            Self::Dir::Down => Self::new(self.x, self.y - 1, self.z),
            Self::Dir::DownSouthWest => Self::new(self.x - 1, self.y - 1, self.z - 1),
            Self::Dir::DownWest => Self::new(self.x - 1, self.y - 1, self.z),
            Self::Dir::DownSouth => Self::new(self.x, self.y - 1, self.z - 1),
        }
    }

    fn range(self, v: isize) -> Self::Iter {
        RangeIter::new(self, v)
    }
}

#[derive(Component, Clone)]
pub struct RddCell {
    _type: Arc<CellType>,
    pub gens: [bool; GENS],
    pub timers: [u8; TIMERS],
    // Могла ли клетка делится в предыдущий тик
    d: bool,
}

impl Cell for RddCell {
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

#[derive(Component, Default)]
pub struct RddController {
    yaw: f32,
    pitch: f32,
}

impl Controller for RddController {
    const SPEED: f32 = 50.0;

    fn update(
        time: Res<Time>,
        kbd: Res<ButtonInput<KeyCode>>,
        camera: Single<(Mut<Transform>, Mut<Self>)>,
        mut cursor: Single<Mut<CursorOptions>, With<PrimaryWindow>>,
        mut mouse: MessageReader<bevy::input::mouse::MouseMotion>,
    ) {
        let (mut camera, mut controller) = camera.into_inner();
        let mut velocity = Vec3::ZERO;

        if kbd.pressed(KeyCode::KeyW) {
            velocity += *camera.forward();
        }
        if kbd.pressed(KeyCode::KeyA) {
            velocity += *camera.left();
        }
        if kbd.pressed(KeyCode::KeyS) {
            velocity += *camera.back();
        }
        if kbd.pressed(KeyCode::KeyD) {
            velocity += *camera.right();
        }

        if kbd.pressed(KeyCode::ControlLeft) {
            velocity -= Vec3::Y * 1.5;
        }
        if kbd.pressed(KeyCode::Space) {
            velocity += Vec3::Y * 1.5;
        }

        if velocity != Vec3::ZERO {
            camera.translation += velocity.normalize() * Self::SPEED * time.delta_secs();
        }

        if kbd.just_pressed(KeyCode::Escape) {
            cursor.grab_mode = match cursor.grab_mode {
                CursorGrabMode::None => CursorGrabMode::Confined,
                _ => CursorGrabMode::None,
            };

            cursor.visible = cursor.grab_mode == CursorGrabMode::None;
        }

        let mut delta = Vec2::ZERO;
        for m in mouse.read() {
            delta += m.delta;
        }

        if cursor.grab_mode != CursorGrabMode::None {
            if delta == Vec2::ZERO {
                return;
            }

            controller.yaw -= delta.x * Self::SENSITIVITY;
            controller.pitch -= delta.y * Self::SENSITIVITY;

            controller.pitch = controller
                .pitch
                .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());

            camera.rotation =
                Quat::from_euler(EulerRot::YXZ, controller.yaw, controller.pitch, 0.0);
        } else {
            let (yaw, pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
            controller.yaw = yaw;
            controller.pitch = pitch;
        }
    }
}

#[derive(Resource)]
pub struct RddMaterials {
    pub mesh: Mesh3d,
    //pub selected_mesh: Mesh3d,
    //pub selected_material: MeshMaterial3d<StandardMaterial>,
    pub materials: HashMap<String, MeshMaterial3d<StandardMaterial>>,
}

impl RddMaterials {
    pub const VERTICES: [[f32; 3]; 14] = [
        [-4.0, 0.0, 0.0],   // 0
        [-2.0, 2.0, -2.0],  // 1
        [0.0, 4.0, 0.0],    // 2
        [-2.0, 2.0, 2.0],   // 3
        [-2.0, -2.0, -2.0], // 4
        [0.0, 0.0, -4.0],   // 5
        [2.0, 2.0, -2.0],   // 6
        [2.0, 2.0, 2.0],    // 7
        [4.0, 0.0, 0.0],    // 8
        [0.0, 0.0, 4.0],    // 9
        [-2.0, -2.0, 2.0],  // 10
        [0.0, -4.0, 0.0],   // 11
        [2.0, -2.0, 2.0],   // 12
        [2.0, -2.0, -2.0],  // 13
    ];

    pub const INDICIES: [u32; 72] = [
        0, 2, 1, 0, 2, 3, // f1
        1, 4, 0, 1, 4, 5, // f2
        2, 5, 1, 2, 5, 6, // f3
        2, 8, 6, 2, 8, 7, // f4
        9, 2, 7, 9, 2, 3, // f5
        10, 3, 0, 10, 3, 9, // f6
        9, 11, 10, 9, 11, 12, // f7
        7, 12, 9, 7, 12, 8, // f8
        10, 4, 0, 10, 4, 11, // f9
        11, 5, 4, 11, 5, 13, // f10
        8, 5, 13, 8, 5, 6, // f11
        8, 11, 12, 8, 11, 13, // f12
    ];

    pub fn raw_mesh() -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        let mut vertices = Vec::with_capacity(Self::VERTICES.len());
        for v in Self::VERTICES {
            vertices.push(Vec3::from(v));
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

        let indices = Vec::from(Self::INDICIES);
        mesh.insert_indices(Indices::U32(indices));

        mesh.duplicate_vertices();
        mesh.compute_flat_normals();

        mesh
    }
}

impl FromWorld for RddMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let mesh = Mesh3d(meshes.add(Self::raw_mesh()));

        let grid = world.get_resource::<RddGrid>().unwrap();
        let config = grid.config.clone();

        let mut color_materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();

        let mut materials = HashMap::with_capacity(config.types.capacity());
        for (name, t) in config.types.iter() {
            let material = color_materials.add(StandardMaterial::from_color(t.color));
            materials.insert(name.clone(), MeshMaterial3d(material));
        }

        Self { mesh, materials }
    }
}

#[derive(Default, Component)]
pub struct RddOrigin;

#[derive(Resource)]
pub struct RddGrid {
    pub config: Arc<Config>,
    parent: Entity,

    _selected: Option<(RddCoords, Entity)>,
    data: HashMap<RddCoords, Entity>,
    current_tick: u8,
}

impl Grid for RddGrid {
    type Cell = RddCell;
    type Coords = RddCoords;
    type Controller = RddController;
    type Materials = RddMaterials;
    type Origin = RddOrigin;

    fn new(parent: Entity, config: Arc<Config>) -> Self {
        let data = HashMap::new();

        Self {
            config,
            parent,
            _selected: None,
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

    fn on_setup(mut commands: Commands) {
        let transform = Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::ZERO);
        commands.spawn((Camera3d::default(), RddController::default(), transform));
        commands.insert_resource(ClearColor(Color::srgb_u8(43, 43, 43)));
    }

    fn on_load(
        mut grid: ResMut<Self>,
        mut concentrations: ResMut<Concentrations<RddCoords>>,
        materials: Res<Self::Materials>,
        mut commands: Commands,
    ) {
        info!("Initializing grid...");

        let coords = Self::Coords::ORIGIN;
        let _type = grid.config.types.get(&grid.config.default).unwrap().clone();
        let mut cell = RddCell::new(_type);
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
        concentrations: &mut Concentrations<RddCoords>,
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
        mut _commands: Commands,
        mut _grid: ResMut<Self>,
        _materials: Res<Self::Materials>,
        _camera: Single<(Ref<Camera>, Ref<GlobalTransform>), With<Self::Controller>>,
        _origin: Single<Ref<Transform>, With<Self::Origin>>,
        _window: Single<Ref<Window>, With<bevy::window::PrimaryWindow>>,
        _msb: Res<ButtonInput<MouseButton>>,
    ) {
        // todo
    }

    fn add_selection(
        &mut self,
        _commands: &mut Commands,
        _materials: &Self::Materials,
        _coords: Self::Coords,
    ) {
        // todo
    }

    fn clear_selection(&mut self, commands: &mut Commands) {
        if let Some((_, entity)) = self._selected {
            commands.entity(entity).despawn();
        }

        self._selected = None;
    }
}
