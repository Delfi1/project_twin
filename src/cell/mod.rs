pub mod parser;

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use std::ops::Mul;
use std::sync::Arc;

pub const SIZE: f32 = 16.0;
// Внутренний радиус гексагона, корень из 3
pub const INNER_RADIUS: f32 = 1.73205;
pub const THINKNESS: f32 = 4.0;

#[derive(Component)]
// Точка отчёта симуляции, если сдвинуть её, сдвинется вся сетка
pub struct Origin;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Coord {
    q: isize,
    r: isize,
    s: isize,
}

impl Coord {
    pub const ORIGIN: Self = Self::new(0, 0);

    pub const fn new(q: isize, r: isize) -> Self {
        Self { q, r, s: -q - r }
    }

    pub fn neighbor(&self, dir: Direction) -> Self {
        use Direction::*;
        match dir {
            None => *self,
            East => Self::new(self.q + 1, self.r),
            West => Self::new(self.q - 1, self.r),
            Northwest => Self::new(self.q, self.r - 1),
            Southeast => Self::new(self.q, self.r + 1),
            Northeast => Self::new(self.q + 1, self.r - 1),
            Southwest => Self::new(self.q - 1, self.r + 1),
        }
    }

    pub fn neighbors(&self) -> impl Iterator<Item = Self> + '_ {
        struct NeighborIter<'a> {
            c: &'a Coord,
            iter: std::slice::Iter<'a, Direction>,
        }
        impl<'a> Iterator for NeighborIter<'a> {
            type Item = Coord;
            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next().map(|d| self.c.neighbor(*d))
            }
        }

        NeighborIter {
            c: self,
            iter: Direction::NEIGHBORS.iter(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Direction {
    #[default]
    None,
    East,
    West,
    Northeast,
    Southwest,
    Northwest,
    Southeast,
}

impl Direction {
    pub const NEIGHBORS: [Self; 6] = [
        Self::East,
        Self::Northeast,
        Self::Southeast,
        Self::West,
        Self::Southwest,
        Self::Northwest,
    ];

    pub fn random() -> Self {
        Self::NEIGHBORS[rand::random_range(0..6)]
    }
}

#[derive(Component, Default)]
pub struct Cell {}

impl Cell {
    pub fn new(_config: &parser::Config) -> Self {
        todo!()
    }
}

#[derive(Resource)]
pub struct HexGrid {
    pub parser: Arc<parser::Config>,
    // Родительский элемент от которого уже отрисовываются все клетки
    parent: Entity,

    mesh: Mesh2d,
    materials: HashMap<String, MeshMaterial2d<ColorMaterial>>,

    grid: HashMap<Coord, Entity>,
}

impl HexGrid {
    pub fn new(
        parent: Entity,
        parser: Arc<parser::Config>,
        meshes: &mut Assets<Mesh>,
        color_materials: &mut Assets<ColorMaterial>,
    ) -> Self {
        let grid = HashMap::new();
        let mesh = Mesh2d(meshes.add(RegularPolygon::new(SIZE, 6).to_ring(THINKNESS)));
        let mut materials = HashMap::with_capacity(parser.types.capacity());
        for (name, t) in parser.types.iter() {
            let material = color_materials.add(Color::Srgba(t.color));
            materials.insert(name.clone(), MeshMaterial2d(material));
        }

        Self {
            parser,
            parent,
            mesh,
            materials,
            grid,
        }
    }

    pub fn get(&self, coords: &Coord) -> Option<&Entity> {
        self.grid.get(coords)
    }

    pub fn insert(&mut self, commands: &mut Commands, coords: Coord) {
        let material = self.materials.get(&self.parser.default).unwrap().clone();
        let mesh = self.mesh.clone();

        let pos = Vec3::new(
            INNER_RADIUS * coords.q as f32 + INNER_RADIUS / 2.0 * coords.r as f32,
            3. / 2. * coords.r as f32,
            0.0,
        )
        .mul(SIZE);

        let entity = commands
            .spawn((
                Cell::default(),
                mesh,
                material,
                Transform::from_translation(pos).with_scale(Vec3::splat(0.9)),
            ))
            .id();
        commands.entity(self.parent).add_child(entity);

        self.grid.insert(coords, entity);
    }

    pub fn iter_neighbors(coords: Coord, range: isize) {
        todo!();
    }
}
