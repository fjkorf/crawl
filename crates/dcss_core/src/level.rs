//! Multi-level dungeon state management with level persistence.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::terrain::TerrainGrid;
use crate::types::Coord;

pub const MAX_DEPTH: i32 = 5;

#[derive(Resource, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CurrentLevel {
    pub branch: String,
    pub depth: i32,
}

impl Default for CurrentLevel {
    fn default() -> Self {
        Self { branch: "D".into(), depth: 1 }
    }
}

#[derive(Resource, Default)]
pub struct StairsAction {
    pub pending: Option<StairsDirection>,
}

#[derive(Debug, Clone, Copy)]
pub enum StairsDirection {
    Down,
    Up,
}

/// Saved state for a single dungeon level.
#[derive(Clone)]
pub struct SavedLevel {
    pub grid: TerrainGrid,
    pub monsters: Vec<SavedMonster>,
    pub items: Vec<SavedItem>,
    pub player_pos: Coord,
}

#[derive(Clone)]
pub struct SavedMonster {
    pub name: String,
    pub pos: Coord,
    pub hp: i32,
    pub max_hp: i32,
    pub ac: i32,
    pub ev: i32,
    pub hd: i32,
    pub speed: i32,
    pub attack_type: String,
    pub attack_damage: i32,
}

#[derive(Clone)]
pub struct SavedItem {
    pub name: String,
    pub pos: Coord,
    pub class: crate::item::ItemClass,
    pub plus: i32,
    pub quantity: i32,
    pub glyph: char,
}

/// Stores saved levels for persistence when moving between floors.
#[derive(Resource, Default)]
pub struct LevelStore {
    pub levels: HashMap<i32, SavedLevel>,
}
