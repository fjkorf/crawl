use bevy::prelude::*;
use std::collections::HashMap;

pub const TILE_SIZE: f32 = 32.0;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum TileId {
    FloorGreyDirt,
    WallBrickDark,
    DoorClosed,
    DoorOpen,
    StairsDown,
    StairsUp,
    PlayerHuman,
    OrbOfZot,
    // Monsters
    MonGoblin,
    MonKobold,
    MonGnoll,
    MonOrc,
    MonRat,
    MonBat,
    MonJackal,
    MonAdder,
    MonHobgoblin,
    MonOgre,
    MonScorpion,
    MonTroll,
    MonYak,
    // Items
    ItemDagger,
    ItemSword,
    ItemAxe,
    ItemMace,
    ItemSpear,
    ItemLeather,
    ItemChainMail,
    ItemPotion,
    ItemScroll,
    ItemGold,
}

#[derive(Resource, Default)]
pub struct TileRegistry {
    pub tiles: HashMap<TileId, Handle<Image>>,
}

impl TileRegistry {
    pub fn get(&self, id: TileId) -> Handle<Image> {
        self.tiles
            .get(&id)
            .cloned()
            .unwrap_or_default()
    }
}

/// Map a monster name (from YAML) to a TileId.
pub fn monster_name_to_tile(name: &str) -> Option<TileId> {
    match name {
        "goblin" => Some(TileId::MonGoblin),
        "kobold" => Some(TileId::MonKobold),
        "gnoll" => Some(TileId::MonGnoll),
        "orc" => Some(TileId::MonOrc),
        "rat" => Some(TileId::MonRat),
        "bat" => Some(TileId::MonBat),
        "jackal" => Some(TileId::MonJackal),
        "adder" => Some(TileId::MonAdder),
        "hobgoblin" => Some(TileId::MonHobgoblin),
        "ogre" => Some(TileId::MonOgre),
        "scorpion" => Some(TileId::MonScorpion),
        "troll" => Some(TileId::MonTroll),
        "yak" => Some(TileId::MonYak),
        _ => Some(TileId::MonGoblin), // fallback
    }
}

/// Map an item subtype to a TileId.
pub fn item_subtype_to_tile(subtype: &str, class: &str) -> TileId {
    match (class, subtype) {
        ("weapon", "dagger") => TileId::ItemDagger,
        ("weapon", "short_sword") | ("weapon", "long_sword") | ("weapon", "broad_sword") => TileId::ItemSword,
        ("weapon", "hand_axe") | ("weapon", "war_axe") | ("weapon", "battle_axe") => TileId::ItemAxe,
        ("weapon", "mace") => TileId::ItemMace,
        ("weapon", "spear") => TileId::ItemSpear,
        ("armour", _) => TileId::ItemLeather,
        ("potion", _) => TileId::ItemPotion,
        ("scroll", _) => TileId::ItemScroll,
        ("gold", _) => TileId::ItemGold,
        _ => TileId::ItemGold,
    }
}

const TILE_PATHS: &[(TileId, &str)] = &[
    (TileId::FloorGreyDirt, "rltiles/dngn/floor/limestone0.png"),
    (TileId::WallBrickDark, "rltiles/dngn/wall/brick_dark_1_0.png"),
    (TileId::DoorClosed, "rltiles/dngn/doors/closed_door.png"),
    (TileId::DoorOpen, "rltiles/dngn/doors/open_door.png"),
    (TileId::StairsDown, "rltiles/dngn/gateways/stone_stairs_down.png"),
    (TileId::StairsUp, "rltiles/dngn/gateways/stone_stairs_up.png"),
    (TileId::PlayerHuman, "rltiles/player/base/human_m.png"),
    (TileId::OrbOfZot, "rltiles/item/misc/orb_of_zot1.png"),
    (TileId::MonGoblin, "rltiles/mon/humanoids/goblin.png"),
    (TileId::MonKobold, "rltiles/mon/humanoids/kobold.png"),
    (TileId::MonGnoll, "rltiles/mon/humanoids/gnoll.png"),
    (TileId::MonOrc, "rltiles/mon/humanoids/orc.png"),
    (TileId::MonRat, "rltiles/mon/animals/rat.png"),
    (TileId::MonBat, "rltiles/mon/animals/bat.png"),
    (TileId::MonJackal, "rltiles/mon/animals/jackal.png"),
    (TileId::MonAdder, "rltiles/mon/animals/adder.png"),
    (TileId::MonHobgoblin, "rltiles/mon/humanoids/hobgoblin.png"),
    (TileId::MonOgre, "rltiles/mon/humanoids/ogre.png"),
    (TileId::MonScorpion, "rltiles/mon/animals/scorpion.png"),
    (TileId::MonTroll, "rltiles/mon/humanoids/troll.png"),
    (TileId::MonYak, "rltiles/mon/animals/yak.png"),
    // Items
    (TileId::ItemDagger, "rltiles/item/weapon/dagger.png"),
    (TileId::ItemSword, "rltiles/item/weapon/short_sword1.png"),
    (TileId::ItemAxe, "rltiles/item/weapon/hand_axe1.png"),
    (TileId::ItemMace, "rltiles/item/weapon/mace1.png"),
    (TileId::ItemSpear, "rltiles/item/weapon/spear1.png"),
    (TileId::ItemLeather, "rltiles/item/armour/leather_armour1.png"),
    (TileId::ItemChainMail, "rltiles/item/armour/chain_mail1.png"),
    (TileId::ItemPotion, "rltiles/item/potion/ruby.png"),
    (TileId::ItemScroll, "rltiles/item/scroll/i-teleportation.png"),
    (TileId::ItemGold, "rltiles/item/gold/01.png"),
];

pub fn load_tiles(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut registry = TileRegistry::default();
    for &(id, path) in TILE_PATHS {
        registry.tiles.insert(id, asset_server.load(path));
    }
    commands.insert_resource(registry);
}
