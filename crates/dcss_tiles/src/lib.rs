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
    PlayerHuman,
    MonGoblin,
    MonKobold,
    MonGnoll,
    MonOrc,
    MonRat,
    MonBat,
    MonJackal,
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
        _ => None,
    }
}

const TILE_PATHS: &[(TileId, &str)] = &[
    (TileId::FloorGreyDirt, "rltiles/dngn/floor/limestone0.png"),
    (TileId::WallBrickDark, "rltiles/dngn/wall/brick_dark_1_0.png"),
    (TileId::DoorClosed, "rltiles/dngn/doors/closed_door.png"),
    (TileId::DoorOpen, "rltiles/dngn/doors/open_door.png"),
    (TileId::StairsDown, "rltiles/dngn/gateways/stone_stairs_down.png"),
    (TileId::PlayerHuman, "rltiles/player/base/human_m.png"),
    (TileId::MonGoblin, "rltiles/mon/humanoids/goblin.png"),
    (TileId::MonKobold, "rltiles/mon/humanoids/kobold.png"),
    (TileId::MonGnoll, "rltiles/mon/humanoids/gnoll.png"),
    (TileId::MonOrc, "rltiles/mon/humanoids/orc.png"),
    (TileId::MonRat, "rltiles/mon/animals/rat.png"),
    (TileId::MonBat, "rltiles/mon/animals/bat.png"),
    (TileId::MonJackal, "rltiles/mon/animals/jackal.png"),
];

pub fn load_tiles(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut registry = TileRegistry::default();
    for &(id, path) in TILE_PATHS {
        registry.tiles.insert(id, asset_server.load(path));
    }
    commands.insert_resource(registry);
}
