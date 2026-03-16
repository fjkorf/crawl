use bevy::prelude::*;
use crate::item::ItemDef;
use crate::types::Coord;

#[derive(Debug, Clone, Default)]
pub struct StatusEffects {
    pub poison: i32,  // damage per turn remaining
    pub might: i32,   // turns of +5 damage
    pub haste: i32,   // turns of double actions
}

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
    pub status: StatusEffects,
    pub regen_counter: i32, // counts turns for HP regen
    pub turns: i32,         // total turns taken
}

impl Default for Player {
    fn default() -> Self {
        Self {
            pos: Coord::new(5, 5),
            hp: 20, max_hp: 20, mp: 5, max_mp: 5,
            ac: 2, ev: 10,
            str_stat: 12, int_stat: 10, dex_stat: 10,
            xl: 1, xp: 0, xp_next: 20, gold: 0,
            turn_is_over: false, time_taken: 0,
            has_orb: false,
            equipped_weapon: None, equipped_armour: None,
            base_damage: 5, base_ac: 0,
            status: StatusEffects::default(),
            regen_counter: 0,
            turns: 0,
        }
    }
}

impl Player {
    pub fn weapon_damage(&self) -> i32 {
        let base = self.equipped_weapon.as_ref()
            .map(|w| w.base_damage + w.plus)
            .unwrap_or(self.base_damage);
        let stat_bonus = self.str_stat / 3;
        let might_bonus = if self.status.might > 0 { 5 } else { 0 };
        base + stat_bonus + might_bonus
    }

    pub fn weapon_brand(&self) -> Option<crate::item::Brand> {
        self.equipped_weapon.as_ref().and_then(|w| w.brand)
    }

    pub fn total_ac(&self) -> i32 {
        self.base_ac
            + self.equipped_armour.as_ref().map(|a| a.base_ac + a.plus).unwrap_or(0)
            + self.ac
    }

    pub fn accuracy(&self) -> i32 {
        10 + self.xl * 2
            + self.equipped_weapon.as_ref().map(|w| w.plus).unwrap_or(0)
    }

    pub fn weapon_name(&self) -> String {
        self.equipped_weapon.as_ref()
            .map(|w| w.name.clone())
            .unwrap_or_else(|| "fists".into())
    }

    pub fn armour_name(&self) -> String {
        self.equipped_armour.as_ref()
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "nothing".into())
    }

    /// Tick status effects. Returns messages to display.
    pub fn tick_status(&mut self) -> Vec<String> {
        let mut msgs = Vec::new();

        if self.status.poison > 0 {
            self.hp -= 1;
            self.status.poison -= 1;
            msgs.push("You are poisoned! (-1 HP)".into());
            if self.status.poison == 0 {
                msgs.push("You are no longer poisoned.".into());
            }
        }

        if self.status.might > 0 {
            self.status.might -= 1;
            if self.status.might == 0 {
                msgs.push("You feel your strength return to normal.".into());
            }
        }

        if self.status.haste > 0 {
            self.status.haste -= 1;
            if self.status.haste == 0 {
                msgs.push("You feel yourself slow down.".into());
            }
        }

        // HP regen: 1 HP per 5 turns if not poisoned
        if self.status.poison == 0 && self.hp < self.max_hp {
            self.regen_counter += 1;
            if self.regen_counter >= 5 {
                self.regen_counter = 0;
                self.hp = (self.hp + 1).min(self.max_hp);
            }
        }

        self.turns += 1;
        msgs
    }
}

#[derive(Component)]
pub struct PlayerSprite;
