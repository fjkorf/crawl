use bevy::prelude::*;
use crate::types::Coord;

#[derive(Resource)]
pub struct Player {
    pub pos: Coord,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub ac: i32,
    pub ev: i32,
    pub str_stat: i32,
    pub int_stat: i32,
    pub dex_stat: i32,
    pub xl: i32,
    pub gold: i32,
    pub turn_is_over: bool,
    pub time_taken: i32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            pos: Coord::new(5, 5),
            hp: 20,
            max_hp: 20,
            mp: 5,
            max_mp: 5,
            ac: 2,
            ev: 10,
            str_stat: 12,
            int_stat: 10,
            dex_stat: 10,
            xl: 1,
            gold: 0,
            turn_is_over: false,
            time_taken: 0,
        }
    }
}

#[derive(Component)]
pub struct PlayerSprite;
