//! Screenshot test binary: renders the dungeon scene for N frames, captures a screenshot, and exits.
//!
//! Usage:
//!   cargo run --example screenshot_test
//!   SCREENSHOT_OUTPUT=path/to/output.png cargo run --example screenshot_test

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

use dcss_core::message::MessageLog;
use dcss_core::monster::*;
use dcss_core::player::{Player, PlayerSprite};
use dcss_core::terrain::{self, Feature, TerrainGrid};
use dcss_core::turn::PendingMove;
use dcss_core::types::*;
use dcss_tiles::{self, TileId, TileRegistry, TILE_SIZE};
use dcss_ui::{message_panel, stat_panel};

#[derive(Resource)]
struct FrameCounter {
    count: u32,
    screenshot_taken: bool,
    output_path: String,
}

fn main() {
    let output_path = std::env::var("SCREENSHOT_OUTPUT")
        .unwrap_or_else(|_| "tests/snapshots/dungeon_scene.new.png".to_string());

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "DCSS Screenshot Test".into(),
                resolution: bevy::window::WindowResolution::new(1280, 960),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .insert_resource(ClearColor(Color::srgb(0.1, 0.0, 0.2)))
        .insert_resource(terrain::hardcoded_dungeon())
        .init_resource::<Player>()
        .init_resource::<MonsterGrid>()
        .init_resource::<MonsterDefs>()
        .init_resource::<MessageLog>()
        .init_resource::<PendingMove>()
        .insert_resource(FrameCounter {
            count: 0,
            screenshot_taken: false,
            output_path,
        })
        // Startup
        .add_systems(Startup, (dcss_tiles::load_tiles, setup_camera))
        .add_systems(
            Startup,
            (load_monster_defs, spawn_dungeon, spawn_player, spawn_monsters, setup_messages)
                .chain()
                .after(dcss_tiles::load_tiles),
        )
        // Screenshot control
        .add_systems(Update, screenshot_controller)
        // Render sync
        .add_systems(Update, (sync_player_sprite, camera_follow))
        // egui panels
        .add_systems(
            EguiPrimaryContextPass,
            (stat_panel::stat_panel_system, message_panel::message_panel_system).chain(),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scale: 0.5,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn setup_messages(mut messages: ResMut<MessageLog>) {
    messages.add("Welcome to the dungeon! Use arrow keys or hjkl to move.");
    messages.add("Walk into monsters to attack them.");
    messages.add("The goblin hisses at you.");
}

fn screenshot_controller(
    mut commands: Commands,
    mut counter: ResMut<FrameCounter>,
    mut exit: MessageWriter<AppExit>,
) {
    counter.count += 1;

    // Wait enough frames for assets to load and scene to render
    if counter.count == 15 && !counter.screenshot_taken {
        let path = counter.output_path.clone();
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
        counter.screenshot_taken = true;
    }

    // Exit after screenshot has had time to save
    if counter.count >= 25 {
        exit.write(AppExit::Success);
    }
}

// --- Dungeon Rendering (duplicated from main.rs) ---

#[derive(Component)]
struct TerrainSprite;

fn spawn_dungeon(mut commands: Commands, terrain_grid: Res<TerrainGrid>, tiles: Res<TileRegistry>) {
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let tile_id = match terrain_grid.cells[y][x] {
                Feature::Wall => TileId::WallBrickDark,
                Feature::Floor => TileId::FloorGreyDirt,
                Feature::ClosedDoor => TileId::DoorClosed,
                Feature::OpenDoor => TileId::DoorOpen,
                Feature::StairsDown => TileId::StairsDown,
            };
            commands.spawn((
                TerrainSprite,
                Sprite::from_image(tiles.get(tile_id)),
                Transform::from_xyz(x as f32 * TILE_SIZE, -(y as f32) * TILE_SIZE, 0.0),
            ));
        }
    }
}

fn spawn_player(mut commands: Commands, player: Res<Player>, tiles: Res<TileRegistry>) {
    let world = player.pos.to_world();
    commands.spawn((
        PlayerSprite,
        Sprite::from_image(tiles.get(TileId::PlayerHuman)),
        Transform::from_xyz(world.x, world.y, 1.0),
    ));
}

fn sync_player_sprite(player: Res<Player>, mut query: Query<&mut Transform, With<PlayerSprite>>) {
    for mut transform in &mut query {
        let world = player.pos.to_world();
        transform.translation.x = world.x;
        transform.translation.y = world.y;
    }
}

fn camera_follow(
    player: Res<Player>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<PlayerSprite>)>,
) {
    for mut transform in &mut camera {
        let target = player.pos.to_world();
        transform.translation.x = target.x;
        transform.translation.y = target.y;
    }
}

fn load_monster_defs(mut defs: ResMut<MonsterDefs>) {
    let base_path = "crawl-ref/source/dat/mons";
    let names = ["goblin", "kobold", "rat", "bat", "jackal", "gnoll", "orc"];
    for name in &names {
        let path = format!("{}/{}.yaml", base_path, name);
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(def) = serde_yaml::from_str::<MonsterDef>(&content) {
                defs.0.insert(name.to_string(), def);
            }
        }
    }
}

fn spawn_monsters(
    mut commands: Commands,
    defs: Res<MonsterDefs>,
    tiles: Res<TileRegistry>,
    mut grid: ResMut<MonsterGrid>,
) {
    let placements = [
        ("goblin", Coord::new(8, 4)),
        ("rat", Coord::new(25, 4)),
        ("kobold", Coord::new(30, 6)),
        ("gnoll", Coord::new(5, 15)),
        ("orc", Coord::new(10, 14)),
        ("jackal", Coord::new(25, 14)),
        ("bat", Coord::new(33, 15)),
    ];

    for (name, pos) in &placements {
        let Some(def) = defs.0.get(*name) else { continue };
        let Some(tile_id) = dcss_tiles::monster_name_to_tile(name) else { continue };

        let hp = def.hp_10x / 10;
        let attack = def
            .attacks
            .first()
            .map(|a| MeleeAttack {
                attack_type: a.attack_type.clone(),
                damage: a.damage,
            })
            .unwrap_or(MeleeAttack {
                attack_type: "hit".to_string(),
                damage: 1,
            });

        let world = pos.to_world();
        let entity = commands
            .spawn((
                MonsterTag,
                MonsterName(def.name.clone()),
                Health { current: hp, max: hp },
                Position(*pos),
                Speed { base: def.speed.unwrap_or(10), energy: 0 },
                attack,
                ArmorClass(def.ac),
                Evasion(def.ev),
                HitDice(def.hd),
                Sprite::from_image(tiles.get(tile_id)),
                Transform::from_xyz(world.x, world.y, 1.0),
            ))
            .id();
        grid.set(*pos, Some(entity));
    }
}
