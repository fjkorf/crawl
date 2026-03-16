//! Item system: definitions, brands, floor items, and player inventory.

use bevy::prelude::*;
use crate::types::Coord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemClass {
    Weapon,
    Armour,
    Potion,
    Scroll,
    Gold,
    Orb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Brand {
    Flaming,   // +2-4 fire damage
    Freezing,  // +2-4 cold damage
    Draining,  // heal attacker 1/4 of damage dealt
    Venom,     // chance to poison
}

impl Brand {
    pub fn name(self) -> &'static str {
        match self {
            Brand::Flaming => "flaming",
            Brand::Freezing => "freezing",
            Brand::Draining => "draining",
            Brand::Venom => "venom",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ItemDef {
    pub name: String,
    pub class: ItemClass,
    pub plus: i32,
    pub base_damage: i32,  // for weapons
    pub base_ac: i32,      // for armour
    pub brand: Option<Brand>,
    pub quantity: i32,
    pub glyph: char,
    pub subtype: String,   // e.g. "short_sword", "leather_armour", "healing"
}

impl Default for ItemDef {
    fn default() -> Self {
        Self {
            name: String::new(), class: ItemClass::Gold, plus: 0,
            base_damage: 0, base_ac: 0, brand: None,
            quantity: 1, glyph: '.', subtype: String::new(),
        }
    }
}

// --- ECS Components ---

#[derive(Component)] pub struct ItemTag;
#[derive(Component)] pub struct ItemName(pub String);
#[derive(Component)] pub struct ItemData(pub ItemDef);
#[derive(Component)] pub struct ItemPosition(pub Coord);

// --- Inventory ---

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

// --- Item Generation ---

pub fn random_item(depth: i32) -> ItemDef {
    use rand::Rng;
    let mut rng = rand::rng();
    let roll = rng.random_range(0..100);

    if roll < 20 {
        // Gold
        ItemDef {
            name: "gold piece".into(), class: ItemClass::Gold,
            quantity: rng.random_range(1..=depth * 5 + 5),
            glyph: '$', subtype: "gold".into(), ..Default::default()
        }
    } else if roll < 40 {
        // Potion
        random_potion(&mut rng)
    } else if roll < 55 {
        // Scroll
        random_scroll(&mut rng)
    } else if roll < 78 {
        // Weapon
        random_weapon(&mut rng, depth)
    } else {
        // Armour
        random_armour(&mut rng, depth)
    }
}

fn random_potion(rng: &mut impl rand::Rng) -> ItemDef {
    let (name, subtype) = match rng.random_range(0..3) {
        0 => ("potion of healing", "healing"),
        1 => ("potion of might", "might"),
        _ => ("potion of haste", "haste"),
    };
    ItemDef {
        name: name.into(), class: ItemClass::Potion,
        glyph: '!', subtype: subtype.into(), ..Default::default()
    }
}

fn random_scroll(rng: &mut impl rand::Rng) -> ItemDef {
    let (name, subtype) = match rng.random_range(0..2) {
        0 => ("scroll of teleportation", "teleportation"),
        _ => ("scroll of blinking", "blinking"),
    };
    ItemDef {
        name: name.into(), class: ItemClass::Scroll,
        glyph: '?', subtype: subtype.into(), ..Default::default()
    }
}

fn random_weapon(rng: &mut impl rand::Rng, depth: i32) -> ItemDef {
    let (base_name, base_dmg, subtype) = if depth <= 2 {
        match rng.random_range(0..3) {
            0 => ("dagger", 4, "dagger"),
            1 => ("hand axe", 7, "hand_axe"),
            _ => ("short sword", 6, "short_sword"),
        }
    } else if depth <= 4 {
        match rng.random_range(0..3) {
            0 => ("long sword", 10, "long_sword"),
            1 => ("mace", 8, "mace"),
            _ => ("spear", 6, "spear"),
        }
    } else {
        match rng.random_range(0..2) {
            0 => ("broad sword", 13, "broad_sword"),
            _ => ("war axe", 11, "war_axe"),
        }
    };

    let plus = rng.random_range(0..=depth / 2);
    let brand = if rng.random_range(0..5) == 0 {
        Some(match rng.random_range(0..4) {
            0 => Brand::Flaming,
            1 => Brand::Freezing,
            2 => Brand::Draining,
            _ => Brand::Venom,
        })
    } else {
        None
    };

    let name = match &brand {
        Some(b) => format!("+{} {} of {}", plus, base_name, b.name()),
        None => format!("+{} {}", plus, base_name),
    };

    ItemDef {
        name, class: ItemClass::Weapon, plus, base_damage: base_dmg,
        brand, glyph: ')', subtype: subtype.into(), ..Default::default()
    }
}

fn random_armour(rng: &mut impl rand::Rng, depth: i32) -> ItemDef {
    let (base_name, base_ac, subtype) = if depth <= 2 {
        match rng.random_range(0..2) {
            0 => ("leather armour", 3, "leather_armour"),
            _ => ("ring mail", 5, "ring_mail"),
        }
    } else if depth <= 4 {
        match rng.random_range(0..2) {
            0 => ("scale mail", 6, "scale_mail"),
            _ => ("chain mail", 8, "chain_mail"),
        }
    } else {
        ("plate armour", 10, "plate_armour")
    };

    let plus = rng.random_range(0..=depth / 2);
    let name = format!("+{} {}", plus, base_name);

    ItemDef {
        name, class: ItemClass::Armour, plus, base_ac,
        glyph: '[', subtype: subtype.into(), ..Default::default()
    }
}
