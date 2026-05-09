pub mod parser;

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use std::ops::Mul;
use std::sync::Arc;

pub const SIZE: f32 = 16.0;
// Внутренний радиус гексагона, корень из 3
pub const INNER_RADIUS: f32 = 1.73205;
pub const THINKNESS: f32 = 4.0;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Coord {
    q: isize,
    r: isize,
    s: isize,
}

impl Coord {
    pub fn new(q: isize, r: isize) -> Self {
        Self { q, r, s: -q - r }
    }

    pub fn origin() -> Self {
        Self { q: 0, r: 0, s: 0 }
    }

    pub fn neighbor(&self, dir: Direction) -> Self {
        use Direction::*;
        match dir {
            None => *self,
            North => Self::new(self.q, self.r - 1),
            South => Self::new(self.q, self.r + 1),
            Northeast => Self::new(self.q + 1, self.r - 1),
            Southwest => Self::new(self.q - 1, self.r + 1),
            Northwest => Self::new(self.q - 1, self.r),
            Southeast => Self::new(self.q + 1, self.r),
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
    North,
    South,
    Northeast,
    Southwest,
    Northwest,
    Southeast,
}

impl Direction {
    pub const NEIGHBORS: [Direction; 6] = [
        Direction::North,
        Direction::Northeast,
        Direction::Southeast,
        Direction::South,
        Direction::Southwest,
        Direction::Northwest,
    ];
}

pub struct Timer {
    index: usize,
    current: u8,
}

impl Timer {
    pub fn tick(&mut self) -> bool {
        self.current -= 1;
        self.current == 0
    }
}

#[derive(Component, Default)]
pub struct Cell {
    gens: [parser::Value; parser::GENS],
    timers: [Option<Timer>; parser::TIMERS],
}

impl Cell {
    pub fn new(_parser: &parser::Parser) -> Self {
        todo!()
    }

    pub fn tick(&mut self, grid: &HexGrid) {
        for mut _t in self.timers.iter_mut() {
            todo!();
        }
    }
}

#[derive(Resource)]
pub struct HexGrid {
    pub parser: Arc<parser::Parser>,
    // Родительский элемент от которого уже отрисовываются все клетки
    parent: Entity,

    mesh: Mesh2d,
    materials: HashMap<String, Handle<ColorMaterial>>,

    grid: HashMap<Coord, Entity>,
}

impl HexGrid {
    pub fn new(
        parent: Entity,
        parser: Arc<parser::Parser>,
        meshes: &mut Assets<Mesh>,
        color_materials: &mut Assets<ColorMaterial>,
    ) -> Self {
        let grid = HashMap::new();
        let mesh = Mesh2d(meshes.add(RegularPolygon::new(SIZE, 6).to_ring(THINKNESS)));
        let mut materials = HashMap::with_capacity(parser.types.capacity());
        for (name, t) in parser.types.iter() {
            materials.insert(name.clone(), color_materials.add(Color::Srgba(t.color)));
        }

        Self {
            parser,
            parent,
            mesh,
            materials,
            grid,
        }
    }

    pub fn add_cell(&mut self, commands: &mut Commands, coords: Coord) {
        let material = MeshMaterial2d(self.materials.get(&self.parser.default).unwrap().clone());
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
                Transform::from_translation(pos),
            ))
            .id();
        commands.entity(self.parent).add_child(entity);

        self.grid.insert(coords, entity);
    }

    pub fn iter_neighbors(coords: Coord, range: isize) {
        todo!();
    }
}
