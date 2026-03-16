use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameMode {
    #[default]
    Play,
    Examine,
    GameOver,
}

/// Pending move command, set by input system, consumed by action system.
#[derive(Resource, Default)]
pub struct PendingMove {
    pub command: Option<(i32, i32)>,
}
