use bevy::prelude::*;
use crate::item::ItemDef;
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
    pub xp: i32,
    pub xp_next: i32,
    pub gold: i32,
    pub turn_is_over: bool,
    pub time_taken: i32,
    pub has_orb: bool,
    pub equipped_weapon: Option<ItemDef>,
    pub equipped_armour: Option<ItemDef>,
    pub base_damage: i32,
    pub base_ac: i32,
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
            xp: 0,
            xp_next: 20,
            gold: 0,
            turn_is_over: false,
            time_taken: 0,
            has_orb: false,
            equipped_weapon: None,
            equipped_armour: None,
            base_damage: 5,
            base_ac: 0,
        }
    }
}

impl Player {
    pub fn weapon_damage(&self) -> i32 {
        self.base_damage + self.str_stat / 3
            + self.equipped_weapon.as_ref().map(|w| w.plus).unwrap_or(0)
    }

    pub fn total_ac(&self) -> i32 {
        self.base_ac + self.equipped_armour.as_ref().map(|a| a.plus + 3).unwrap_or(0) + self.ac
    }

    pub fn accuracy(&self) -> i32 {
        10 + self.xl * 2 + self.equipped_weapon.as_ref().map(|w| w.plus).unwrap_or(0)
    }
}

#[derive(Component)]
pub struct PlayerSprite;
