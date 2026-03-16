use bevy::prelude::*;

/// Top-level game phase: character creation → playing.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GamePhase {
    #[default]
    CharacterCreation,
    Playing,
}

/// In-game mode during the Playing phase.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameMode {
    #[default]
    Play,
    Examine,
    Inventory,
    Help,
    GameOver,
    Won,
}

/// Pending move command, set by input system, consumed by action system.
#[derive(Resource, Default)]
pub struct PendingMove {
    pub command: Option<(i32, i32)>,
}
