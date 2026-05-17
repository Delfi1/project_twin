use bevy::ecs::component::Component;

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

#[derive(Clone, Component, Debug, Copy, Hash, PartialEq, Eq)]
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

    fn to_cartesian(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3::new((2. * y - z) / SQRT2, (-2. * x + z) / SQRT2, z)
    }
}

impl Coords for RddCoords {
    type Dir = RddDirection;
    type Iter = RangeIter;

    fn to_world(&self) -> Vec3 {
        let x = self.x as f32;
        let y = self.y as f32;
        let z = self.z as f32;

        Self::to_cartesian(x, y, z)
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
            Self::Dir::UpEast => Self::new(self.x + 1, self.y, self.z + 1),

            Self::Dir::Down => Self::new(self.x, self.y - 1, self.z),
            Self::Dir::DownSouthWest => Self::new(self.x - 1, self.y - 1, self.z - 1),
            Self::Dir::DownWest => Self::new(self.x - 1, self.y, self.z - 1),
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
    const SPEED: f32 = 20.0;

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
    pub hover_material: MeshMaterial3d<StandardMaterial>,
    pub selected_material: MeshMaterial3d<StandardMaterial>,
    pub materials: HashMap<String, MeshMaterial3d<StandardMaterial>>,
}

impl RddMaterials {
    pub const U: f32 = 0.25;
    pub const U2: f32 = 0.5;
    pub const U3: f32 = 0.75;
    pub const U4: f32 = 1.0;

    pub const VERTICES: [[f32; 3]; 14] = [
        [-Self::U2, -Self::U2, 0.0],       // 0
        [-Self::U3, -Self::U, -Self::U2],  // 1
        [-Self::U2, Self::U2, 0.0],        // 2
        [-Self::U, Self::U, Self::U2],     // 3
        [-Self::U, -Self::U3, -Self::U2],  // 4
        [-Self::U2, -Self::U2, -Self::U4], // 5
        [-Self::U, Self::U, -Self::U2],    // 6
        [Self::U, Self::U3, Self::U2],     // 7
        [Self::U2, Self::U2, 0.0],         // 8
        [Self::U2, Self::U2, Self::U4],    // 9
        [Self::U, -Self::U, Self::U2],     // 10
        [Self::U2, -Self::U2, 0.0],        // 11
        [Self::U3, Self::U, Self::U2],     // 12
        [Self::U, -Self::U, -Self::U2],    // 13
    ];

    pub const INDICIES: [u32; 72] = [
        9, 0, 10, 9, 3, 0, // f1
        9, 2, 3, 9, 7, 2, // f2
        9, 8, 7, 9, 12, 8, // f3
        9, 11, 12, 9, 10, 11, // f4
        0, 11, 10, 11, 0, 4, // f5
        2, 8, 6, 8, 2, 7, // f6
        0, 2, 1, 0, 3, 2, // f7
        12, 11, 8, 8, 11, 13, // f8
        1, 5, 0, 0, 5, 4, // f9
        5, 2, 6, 2, 5, 1, // f10
        5, 8, 13, 8, 5, 6, // f11
        5, 11, 4, 11, 5, 13, // f12
    ];

    pub fn raw_mesh() -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        let mut vertices = Vec::with_capacity(Self::VERTICES.len());
        for [x, y, z] in Self::VERTICES {
            vertices.push(RddCoords::to_cartesian(x, y, z));
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_indices(Indices::U32(Self::INDICIES.into()));

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

        let hover = StandardMaterial::from_color(Color::srgb_u8(0, 99, 177));
        let hover_material = MeshMaterial3d(color_materials.add(hover));
        let selected = StandardMaterial::from_color(Color::srgb_u8(0, 120, 215));
        let selected_material = MeshMaterial3d(color_materials.add(selected));

        Self {
            mesh,
            hover_material,
            selected_material,
            materials,
        }
    }
}

impl Materials for RddMaterials {
    type Mesh = Mesh3d;
    type Material = MeshMaterial3d<StandardMaterial>;

    fn mesh(&self) -> Self::Mesh {
        self.mesh.clone()
    }

    fn material(&self, _type: Arc<CellType>) -> Self::Material {
        self.materials.get(&_type.name).cloned().unwrap()
    }

    fn hovered_material(&self) -> Self::Material {
        self.hover_material.clone()
    }

    fn selected_mesh(&self) -> Self::Mesh {
        self.mesh.clone()
    }

    fn selected_material(&self) -> Self::Material {
        self.selected_material.clone()
    }
}

#[derive(Default, Component)]
pub struct RddOrigin;

#[derive(Resource)]
pub struct RddGrid {
    pub config: Arc<Config>,
    parent: Entity,

    selected: Option<Entity>,
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
            selected: None,
            data,
            current_tick: 128,
        }
    }

    fn get_parent(&self) -> Entity {
        self.parent
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
        commands.spawn((
            DirectionalLight::default(),
            Transform::from_translation(Vec3::new(5.0, 2.0, 5.0)).looking_at(Vec3::ZERO, Vec3::Y),
        ));
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
        let mut cell = RddCell::new(_type.clone());
        cell.gens[0] = true;

        //let c2 = coords.neighbor(&RddDirection::Up);
        grid.insert(&mut commands, &mut concentrations, &materials, coords, cell);

        //let cl2 = RddCell::new(_type);
        //grid.insert(&mut commands, &mut concentrations, &materials, c2, cl2);
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
        let material = materials.material(cell._type.clone());
        let pos = coords.to_world();

        let entity = commands
            .spawn((
                cell,
                coords,
                materials.mesh(),
                material,
                Transform::from_translation(pos),
                Pickable::default(),
            ))
            .observe(Self::on_select)
            .observe(Self::on_hover)
            .observe(Self::on_out)
            .id();

        commands.entity(self.parent).add_child(entity);

        concentrations.insert(coords, [self.current_tick - 2; GENS]);
        self.data.insert(coords, entity)
    }

    fn get_selected(&self) -> Option<&Entity> {
        self.selected.as_ref()
    }

    fn take_selected(&mut self) -> Option<Entity> {
        self.selected.take()
    }

    fn on_select(
        ev: On<Pointer<Press>>,
        mut grid: ResMut<Self>,
        mut commands: Commands,
        _: Query<Ref<Self::Coords>>,
        cells: Query<Ref<Self::Cell>>,
        materials: Res<Self::Materials>,
    ) {
        let entity = ev.event_target();
        let cell = cells.get(entity).unwrap();

        if let Some(prev) = grid.take_selected() {
            commands
                .entity(prev)
                .insert(materials.material(cell._type.clone()));
        }

        commands
            .entity(entity)
            .insert((materials.selected_material(), materials.selected_mesh()));
        grid.selected = Some(entity);
    }

    fn unselect(
        mut commands: Commands,
        mut grid: ResMut<Self>,
        cells: Query<Ref<Self::Cell>>,
        msb: Res<ButtonInput<MouseButton>>,
        materials: Res<Self::Materials>,
    ) {
        if msb.just_pressed(MouseButton::Right) {
            if let Some(prev) = grid.take_selected() {
                let cell = cells.get(prev).unwrap();

                commands
                    .entity(prev)
                    .insert(materials.material(cell._type.clone()));
            }
        }
    }

    fn get_config(&self) -> Arc<Config> {
        self.config.clone()
    }
}
