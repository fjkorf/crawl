use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use crate::types::Coord;

// --- Serde structs matching YAML schema ---

#[derive(Debug, Clone, Deserialize)]
pub struct MonsterDef {
    pub name: String,
    pub glyph: GlyphDef,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub exp: i32,
    #[serde(default)]
    pub will: i32,
    #[serde(default)]
    pub attacks: Vec<AttackDef>,
    #[serde(default = "default_hd")]
    pub hd: i32,
    #[serde(default = "default_hp")]
    pub hp_10x: i32,
    #[serde(default)]
    pub ac: i32,
    #[serde(default = "default_ev")]
    pub ev: i32,
    #[serde(default)]
    pub speed: Option<i32>,
    #[serde(default)]
    pub intelligence: Option<String>,
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default)]
    pub shape: Option<String>,
    #[serde(default)]
    pub shout: Option<String>,
    #[serde(default)]
    pub uses: Option<String>,
    #[serde(default)]
    pub has_corpse: Option<bool>,
}

fn default_hd() -> i32 { 1 }
fn default_hp() -> i32 { 40 }
fn default_ev() -> i32 { 10 }

#[derive(Debug, Clone, Deserialize)]
pub struct GlyphDef {
    pub char: String,
    pub colour: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AttackDef {
    #[serde(rename = "type")]
    pub attack_type: String,
    pub damage: i32,
    #[serde(default)]
    pub flavour: Option<String>,
}

// --- Bevy Resources ---

#[derive(Resource, Default)]
pub struct MonsterDefs(pub HashMap<String, MonsterDef>);

// --- ECS Components ---

#[derive(Component)]
pub struct MonsterTag;

#[derive(Component)]
pub struct MonsterName(pub String);

#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Component)]
pub struct Position(pub Coord);

#[derive(Component)]
pub struct Speed {
    pub base: i32,
    pub energy: i32,
}

#[derive(Component)]
pub struct MeleeAttack {
    pub attack_type: String,
    pub damage: i32,
}

#[derive(Component)]
pub struct ArmorClass(pub i32);

#[derive(Component)]
pub struct Evasion(pub i32);

#[derive(Component)]
pub struct HitDice(pub i32);

// --- Monster Grid ---

use crate::types::MAP_WIDTH;
use crate::types::MAP_HEIGHT;

#[derive(Resource)]
pub struct MonsterGrid {
    pub cells: [[Option<Entity>; MAP_WIDTH]; MAP_HEIGHT],
}

impl Default for MonsterGrid {
    fn default() -> Self {
        Self {
            cells: [[None; MAP_WIDTH]; MAP_HEIGHT],
        }
    }
}

impl MonsterGrid {
    pub fn get(&self, pos: Coord) -> Option<Entity> {
        if pos.in_bounds() {
            self.cells[pos.y as usize][pos.x as usize]
        } else {
            None
        }
    }

    pub fn set(&mut self, pos: Coord, entity: Option<Entity>) {
        if pos.in_bounds() {
            self.cells[pos.y as usize][pos.x as usize] = entity;
        }
    }
}
