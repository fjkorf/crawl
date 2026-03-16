use bevy::prelude::*;
use std::ops::Add;

pub const MAP_WIDTH: usize = 40;
pub const MAP_HEIGHT: usize = 20;
pub const TILE_SIZE: f32 = 32.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn in_bounds(self) -> bool {
        self.x >= 0
            && self.y >= 0
            && (self.x as usize) < MAP_WIDTH
            && (self.y as usize) < MAP_HEIGHT
    }

    pub fn adjacent_to(self, other: Coord) -> bool {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        dx <= 1 && dy <= 1 && (dx + dy) > 0
    }

    pub fn to_world(self) -> Vec3 {
        Vec3::new(
            self.x as f32 * TILE_SIZE,
            -(self.y as f32) * TILE_SIZE,
            0.0,
        )
    }
}

impl Add for Coord {
    type Output = Coord;
    fn add(self, rhs: Coord) -> Coord {
        Coord {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorRef {
    Player,
    Monster(Entity),
}
