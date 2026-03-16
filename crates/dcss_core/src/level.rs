//! Multi-level dungeon state management.

use bevy::prelude::*;
use crate::types::Coord;

/// Identifies the current dungeon level.
#[derive(Resource, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CurrentLevel {
    pub branch: String,
    pub depth: i32,
}

impl Default for CurrentLevel {
    fn default() -> Self {
        Self {
            branch: "D".into(),
            depth: 1,
        }
    }
}

/// Event fired when the player takes stairs.
#[derive(Resource, Default)]
pub struct StairsAction {
    pub pending: Option<StairsDirection>,
}

#[derive(Debug, Clone, Copy)]
pub enum StairsDirection {
    Down,
    Up,
}
