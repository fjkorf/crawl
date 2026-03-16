//! Save/load game state to JSON file.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::item::{ItemClass, Brand, ItemDef, Inventory};
use crate::level::{CurrentLevel, SavedLevel, SavedMonster, SavedItem};
use crate::player::{Player, StatusEffects};
use crate::terrain::{Feature, TerrainGrid};
use crate::fov::VisibilityMap;
use crate::types::*;

const SAVE_FILE: &str = "save.json";

#[derive(Serialize, Deserialize)]
pub struct SaveGame {
    pub player: PlayerSave,
    pub level: LevelSave,
    pub level_store: HashMap<i32, LevelDataSave>,
    pub inventory: InventorySave,
    pub messages: Vec<String>,
    pub visibility: VisibilitySave,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerSave {
    pub pos_x: i32, pub pos_y: i32,
    pub hp: i32, pub max_hp: i32, pub mp: i32, pub max_mp: i32,
    pub ac: i32, pub ev: i32,
    pub str_stat: i32, pub int_stat: i32, pub dex_stat: i32,
    pub xl: i32, pub xp: i32, pub xp_next: i32, pub gold: i32,
    pub has_orb: bool,
    pub equipped_weapon: Option<ItemSave>,
    pub equipped_armour: Option<ItemSave>,
    pub base_damage: i32, pub base_ac: i32,
    pub poison: i32, pub might: i32, pub haste: i32,
    pub regen_counter: i32, pub turns: i32, pub kills: i32,
    pub species_name: String, pub job_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct LevelSave {
    pub branch: String,
    pub depth: i32,
}

#[derive(Serialize, Deserialize)]
pub struct LevelDataSave {
    pub grid: Vec<Vec<u8>>, // Feature as u8
    pub monsters: Vec<MonsterSave>,
    pub items: Vec<ItemOnFloorSave>,
    pub player_pos_x: i32, pub player_pos_y: i32,
}

#[derive(Serialize, Deserialize)]
pub struct MonsterSave {
    pub name: String, pub x: i32, pub y: i32,
    pub hp: i32, pub max_hp: i32, pub ac: i32, pub ev: i32,
    pub hd: i32, pub speed: i32,
    pub attack_type: String, pub attack_damage: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ItemSave {
    pub name: String, pub class: String, pub plus: i32,
    pub base_damage: i32, pub base_ac: i32,
    pub brand: Option<String>,
    pub quantity: i32, pub glyph: char, pub subtype: String,
}

#[derive(Serialize, Deserialize)]
pub struct ItemOnFloorSave {
    pub item: ItemSave,
    pub x: i32, pub y: i32,
}

#[derive(Serialize, Deserialize)]
pub struct InventorySave {
    pub items: Vec<ItemSave>,
    pub gold: i32,
}

#[derive(Serialize, Deserialize)]
pub struct VisibilitySave {
    pub explored: Vec<Vec<bool>>,
}

// --- Conversion helpers ---

impl From<&ItemDef> for ItemSave {
    fn from(item: &ItemDef) -> Self {
        ItemSave {
            name: item.name.clone(),
            class: match item.class { ItemClass::Weapon => "weapon", ItemClass::Armour => "armour",
                ItemClass::Potion => "potion", ItemClass::Scroll => "scroll",
                ItemClass::Gold => "gold", ItemClass::Orb => "orb" }.into(),
            plus: item.plus, base_damage: item.base_damage, base_ac: item.base_ac,
            brand: item.brand.map(|b| b.name().to_string()),
            quantity: item.quantity, glyph: item.glyph, subtype: item.subtype.clone(),
        }
    }
}

impl ItemSave {
    pub fn to_item_def(&self) -> ItemDef {
        let class = match self.class.as_str() {
            "weapon" => ItemClass::Weapon, "armour" => ItemClass::Armour,
            "potion" => ItemClass::Potion, "scroll" => ItemClass::Scroll,
            "gold" => ItemClass::Gold, "orb" => ItemClass::Orb,
            _ => ItemClass::Gold,
        };
        let brand = self.brand.as_deref().and_then(|b| match b {
            "flaming" => Some(Brand::Flaming), "freezing" => Some(Brand::Freezing),
            "draining" => Some(Brand::Draining), "venom" => Some(Brand::Venom),
            _ => None,
        });
        ItemDef {
            name: self.name.clone(), class, plus: self.plus,
            base_damage: self.base_damage, base_ac: self.base_ac,
            brand, quantity: self.quantity, glyph: self.glyph, subtype: self.subtype.clone(),
        }
    }
}

fn feature_to_u8(f: Feature) -> u8 {
    match f { Feature::Wall => 0, Feature::Floor => 1, Feature::ClosedDoor => 2,
              Feature::OpenDoor => 3, Feature::StairsDown => 4, Feature::StairsUp => 5 }
}

fn u8_to_feature(v: u8) -> Feature {
    match v { 1 => Feature::Floor, 2 => Feature::ClosedDoor, 3 => Feature::OpenDoor,
              4 => Feature::StairsDown, 5 => Feature::StairsUp, _ => Feature::Wall }
}

// --- Save ---

pub fn create_save(player: &Player, level: &CurrentLevel, store: &crate::level::LevelStore,
    inventory: &Inventory, messages: &crate::message::MessageLog, vis: &VisibilityMap,
    terrain: &TerrainGrid) -> SaveGame {

    let player_save = PlayerSave {
        pos_x: player.pos.x, pos_y: player.pos.y,
        hp: player.hp, max_hp: player.max_hp, mp: player.mp, max_mp: player.max_mp,
        ac: player.ac, ev: player.ev,
        str_stat: player.str_stat, int_stat: player.int_stat, dex_stat: player.dex_stat,
        xl: player.xl, xp: player.xp, xp_next: player.xp_next, gold: player.gold,
        has_orb: player.has_orb,
        equipped_weapon: player.equipped_weapon.as_ref().map(ItemSave::from),
        equipped_armour: player.equipped_armour.as_ref().map(ItemSave::from),
        base_damage: player.base_damage, base_ac: player.base_ac,
        poison: player.status.poison, might: player.status.might, haste: player.status.haste,
        regen_counter: player.regen_counter, turns: player.turns, kills: player.kills,
        species_name: player.species_name.clone(), job_name: player.job_name.clone(),
    };

    let level_save = LevelSave { branch: level.branch.clone(), depth: level.depth };

    // Save current level into store format too
    let mut all_levels: HashMap<i32, LevelDataSave> = HashMap::new();
    for (&depth, saved) in &store.levels {
        all_levels.insert(depth, saved_level_to_save(saved));
    }
    // Save current level terrain
    all_levels.insert(level.depth, LevelDataSave {
        grid: (0..MAP_HEIGHT).map(|y| (0..MAP_WIDTH).map(|x| feature_to_u8(terrain.cells[y][x])).collect()).collect(),
        monsters: Vec::new(), // current level monsters saved separately via ECS
        items: Vec::new(),
        player_pos_x: player.pos.x, player_pos_y: player.pos.y,
    });

    let inv_save = InventorySave {
        items: inventory.items.iter().map(ItemSave::from).collect(),
        gold: inventory.gold,
    };

    let vis_save = VisibilitySave {
        explored: (0..MAP_HEIGHT).map(|y| (0..MAP_WIDTH).map(|x| vis.explored[y][x]).collect()).collect(),
    };

    let msgs: Vec<String> = messages.messages.iter().rev().take(100).cloned().collect::<Vec<_>>().into_iter().rev().collect();

    SaveGame { player: player_save, level: level_save, level_store: all_levels, inventory: inv_save, messages: msgs, visibility: vis_save }
}

fn saved_level_to_save(saved: &SavedLevel) -> LevelDataSave {
    LevelDataSave {
        grid: (0..MAP_HEIGHT).map(|y| (0..MAP_WIDTH).map(|x| feature_to_u8(saved.grid.cells[y][x])).collect()).collect(),
        monsters: saved.monsters.iter().map(|m| MonsterSave {
            name: m.name.clone(), x: m.pos.x, y: m.pos.y,
            hp: m.hp, max_hp: m.max_hp, ac: m.ac, ev: m.ev,
            hd: m.hd, speed: m.speed, attack_type: m.attack_type.clone(), attack_damage: m.attack_damage,
        }).collect(),
        items: saved.items.iter().map(|i| ItemOnFloorSave {
            item: ItemSave { name: i.name.clone(), class: match i.class {
                ItemClass::Weapon => "weapon", ItemClass::Armour => "armour",
                ItemClass::Potion => "potion", ItemClass::Scroll => "scroll",
                ItemClass::Gold => "gold", ItemClass::Orb => "orb",
            }.into(), plus: i.plus, base_damage: 0, base_ac: 0, brand: None,
                quantity: i.quantity, glyph: i.glyph, subtype: String::new() },
            x: i.pos.x, y: i.pos.y,
        }).collect(),
        player_pos_x: saved.player_pos.x, player_pos_y: saved.player_pos.y,
    }
}

pub fn save_to_file(save: &SaveGame) -> Result<(), String> {
    let json = serde_json::to_string_pretty(save).map_err(|e| format!("serialize: {}", e))?;
    std::fs::write(SAVE_FILE, json).map_err(|e| format!("write: {}", e))
}

pub fn load_from_file() -> Result<SaveGame, String> {
    let json = std::fs::read_to_string(SAVE_FILE).map_err(|e| format!("read: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("deserialize: {}", e))
}

pub fn save_exists() -> bool {
    std::path::Path::new(SAVE_FILE).exists()
}

pub fn delete_save() {
    let _ = std::fs::remove_file(SAVE_FILE);
}

/// Restore player state from a save.
pub fn restore_player(save: &PlayerSave) -> Player {
    Player {
        pos: Coord::new(save.pos_x, save.pos_y),
        hp: save.hp, max_hp: save.max_hp, mp: save.mp, max_mp: save.max_mp,
        ac: save.ac, ev: save.ev,
        str_stat: save.str_stat, int_stat: save.int_stat, dex_stat: save.dex_stat,
        xl: save.xl, xp: save.xp, xp_next: save.xp_next, gold: save.gold,
        has_orb: save.has_orb,
        equipped_weapon: save.equipped_weapon.as_ref().map(|i| i.to_item_def()),
        equipped_armour: save.equipped_armour.as_ref().map(|i| i.to_item_def()),
        base_damage: save.base_damage, base_ac: save.base_ac,
        status: StatusEffects { poison: save.poison, might: save.might, haste: save.haste },
        regen_counter: save.regen_counter, turns: save.turns,
        kills: save.kills, species_name: save.species_name.clone(), job_name: save.job_name.clone(),
        ..Default::default()
    }
}

pub fn restore_terrain(grid_data: &[Vec<u8>]) -> TerrainGrid {
    let mut cells = [[Feature::Wall; MAP_WIDTH]; MAP_HEIGHT];
    for (y, row) in grid_data.iter().enumerate() {
        for (x, &v) in row.iter().enumerate() {
            if y < MAP_HEIGHT && x < MAP_WIDTH {
                cells[y][x] = u8_to_feature(v);
            }
        }
    }
    TerrainGrid { cells }
}

pub fn restore_visibility(vis_data: &VisibilitySave) -> VisibilityMap {
    let mut vis = VisibilityMap::default();
    for (y, row) in vis_data.explored.iter().enumerate() {
        for (x, &v) in row.iter().enumerate() {
            if y < MAP_HEIGHT && x < MAP_WIDTH {
                vis.explored[y][x] = v;
            }
        }
    }
    vis
}
