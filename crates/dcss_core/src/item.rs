//! Item system: definitions, floor items, and player inventory.

use bevy::prelude::*;
use crate::types::Coord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemClass {
    Weapon,
    Armour,
    Potion,
    Scroll,
    Gold,
}

#[derive(Debug, Clone)]
pub struct ItemDef {
    pub name: String,
    pub class: ItemClass,
    pub plus: i32,       // +hit/+dam for weapons, +AC for armour
    pub quantity: i32,
    pub glyph: char,
}

// --- ECS Components ---

#[derive(Component)]
pub struct ItemTag;

#[derive(Component)]
pub struct ItemName(pub String);

#[derive(Component)]
pub struct ItemData(pub ItemDef);

#[derive(Component)]
pub struct ItemPosition(pub Coord);

// --- Player Inventory ---

#[derive(Resource, Default)]
pub struct Inventory {
    pub items: Vec<ItemDef>,
    pub gold: i32,
}

impl Inventory {
    pub fn add(&mut self, item: ItemDef) {
        if item.class == ItemClass::Gold {
            self.gold += item.quantity;
        } else {
            self.items.push(item);
        }
    }
}

/// Simple floor item generation.
pub fn random_item(depth: i32) -> ItemDef {
    use rand::Rng;
    let mut rng = rand::rng();
    let roll = rng.random_range(0..100);

    if roll < 30 {
        ItemDef {
            name: "gold piece".into(),
            class: ItemClass::Gold,
            plus: 0,
            quantity: rng.random_range(1..=depth * 5 + 5),
            glyph: '$',
        }
    } else if roll < 55 {
        ItemDef {
            name: "potion of healing".into(),
            class: ItemClass::Potion,
            plus: 0,
            quantity: 1,
            glyph: '!',
        }
    } else if roll < 70 {
        ItemDef {
            name: "scroll of teleportation".into(),
            class: ItemClass::Scroll,
            plus: 0,
            quantity: 1,
            glyph: '?',
        }
    } else if roll < 85 {
        let plus = rng.random_range(0..=depth / 3);
        ItemDef {
            name: format!("+{} short sword", plus),
            class: ItemClass::Weapon,
            plus,
            quantity: 1,
            glyph: ')',
        }
    } else {
        let plus = rng.random_range(0..=depth / 3);
        ItemDef {
            name: format!("+{} leather armour", plus),
            class: ItemClass::Armour,
            plus,
            quantity: 1,
            glyph: '[',
        }
    }
}
