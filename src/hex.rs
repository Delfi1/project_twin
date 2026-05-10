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
    pub const ORIGIN: Self = Self::cubic(0, 0);

    pub const fn cubic(q: isize, r: isize) -> Self {
        Self { q, r, s: -q - r }
    }
}

impl Coords for HexCoords {
    type Dir = HexDirection;

    fn new(q: isize, r: isize, s: isize) -> Self {
        Self { q, r, s }
    }

    fn neighbor(&self, dir: Self::Dir) -> Self {
        match dir {
            Self::Dir::None => *self,
            Self::Dir::East => Self::cubic(self.q + 1, self.r),
            Self::Dir::West => Self::cubic(self.q - 1, self.r),
            Self::Dir::Northwest => Self::cubic(self.q, self.r - 1),
            Self::Dir::Southeast => Self::cubic(self.q, self.r + 1),
            Self::Dir::Northeast => Self::cubic(self.q + 1, self.r - 1),
            Self::Dir::Southwest => Self::cubic(self.q - 1, self.r + 1),
        }
    }
}

pub const SIZE: f32 = 16.0;
// Внутренний радиус гексагона, корень из 3
pub const INNER_RADIUS: f32 = 1.73205;
pub const THINKNESS: f32 = 4.0;

#[derive(Component, Default)]
pub struct HexCell;

impl Cell for HexCell {}

#[derive(Component)]
pub struct HexController;

impl Controller for HexController {
    // Простой контроллер 2d камеры
    fn update(
        time: Res<Time>,
        mut scroll: Local<f32>,
        kbd: Res<ButtonInput<KeyCode>>,
        mut scroll_msg: MessageReader<bevy::input::mouse::MouseWheel>,
        camera: Single<(Mut<Transform>, Mut<Projection>), With<Camera>>,
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
pub struct HexGrid {
    pub config: Arc<Config>,
    // Родительский элемент от которого уже отрисовываются все клетки
    parent: Entity,
    mesh: Mesh2d,
    materials: Vec<MeshMaterial2d<ColorMaterial>>,

    grid: HashMap<HexCoords, Entity>,
}

impl Grid for HexGrid {
    type Cell = HexCell;
    type Coords = HexCoords;
    type Controller = HexController;

    fn new(
        parent: Entity,
        config: Arc<Config>,
        meshes: &mut Assets<Mesh>,
        color_materials: &mut Assets<ColorMaterial>,
    ) -> Self {
        let grid = HashMap::new();

        let mesh = Mesh2d(meshes.add(RegularPolygon::new(SIZE, 6).to_ring(THINKNESS)));
        let mut materials = Vec::with_capacity(config.types.capacity());
        for t in config.types.iter() {
            let material = color_materials.add(Color::Srgba(t.color));
            materials.push(MeshMaterial2d(material));
        }

        Self {
            config,
            parent,
            mesh,
            materials,
            grid,
        }
    }

    fn insert(&mut self, commands: &mut Commands, coords: Self::Coords, cell: Self::Cell) {
        let material = self.materials[0].clone();
        let mesh = self.mesh.clone();

        let pos = Vec3::new(
            INNER_RADIUS * coords.q as f32 + INNER_RADIUS / 2.0 * coords.r as f32,
            3. / 2. * coords.r as f32,
            0.0,
        )
        .mul(SIZE);

        let entity = commands
            .spawn((
                cell,
                mesh,
                material,
                Transform::from_translation(pos).with_scale(Vec3::splat(0.9)),
            ))
            .id();
        commands.entity(self.parent).add_child(entity);

        self.grid.insert(coords, entity);
    }

    fn get(&self, coords: &Self::Coords) -> Option<&Entity> {
        self.grid.get(coords)
    }

    fn neighbors(&self, _coords: Self::Coords) -> u8 {
        todo!()
    }

    /// 2д мир с белым фоном
    fn on_setup(mut commands: Commands) {
        commands.spawn(Camera2d);
        commands.insert_resource(ClearColor(Color::WHITE));
    }

    fn on_load(mut grid: ResMut<Self>, mut commands: Commands) {
        info!("Initializing viewer...");

        let mut coords = Self::Coords::ORIGIN;
        let mut direction = HexDirection::None;

        // Test generate hexes
        let mut i = 0;
        while i != 12 {
            coords = coords.neighbor(direction);
            if grid.get(&coords).is_none() {
                grid.insert(&mut commands, coords, Self::Cell::default());
                i += 1;
            }

            direction = Direction::random();
        }
    }

    fn on_tick(grid: Res<Self>) {}
}
