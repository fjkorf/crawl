//! DcssGamePlugin — shared game setup used by both the main binary and test examples.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use dcss_core::combat;
use dcss_core::message::MessageLog;
use dcss_core::monster::*;
use dcss_core::player::{Player, PlayerSprite};
use dcss_core::terrain::{self, Feature, TerrainGrid, TerrainSpriteGrid};
use dcss_core::turn::{GameMode, PendingMove};
use dcss_core::types::*;
use dcss_tiles::{self, TileId, TileRegistry, TILE_SIZE};
use dcss_ui::{examine, message_panel, stat_panel};

pub struct DcssGamePlugin;

impl Plugin for DcssGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameMode>()
            .insert_resource(ClearColor(Color::srgb(0.1, 0.0, 0.2)))
            .insert_resource(terrain::hardcoded_dungeon())
            .init_resource::<Player>()
            .init_resource::<MonsterGrid>()
            .init_resource::<MonsterDefs>()
            .init_resource::<MessageLog>()
            .init_resource::<PendingMove>()
            .init_resource::<TerrainSpriteGrid>()
            .init_resource::<examine::ExamineCursor>()
            .init_resource::<examine::MonsterInfoState>()
            // Startup
            .add_systems(Startup, (dcss_tiles::load_tiles, setup_camera))
            .add_systems(
                Startup,
                (load_monster_defs, spawn_dungeon, spawn_player, spawn_monsters, spawn_examine_cursor, welcome_message)
                    .chain()
                    .after(dcss_tiles::load_tiles),
            )
            // Play mode: input → action → world reaction → render sync
            .add_systems(Update, player_input.run_if(in_state(GameMode::Play)))
            .add_systems(
                Update,
                (
                    execute_player_action,
                    check_monster_death,
                    monster_retaliation,
                    check_player_death,
                )
                    .chain()
                    .run_if(in_state(GameMode::Play))
                    .after(player_input),
            )
            .add_systems(
                Update,
                (sync_player_sprite, camera_follow)
                    .run_if(in_state(GameMode::Play))
                    .after(execute_player_action),
            )
            // Play mode: x key enters examine mode
            .add_systems(
                Update,
                enter_examine_mode.run_if(in_state(GameMode::Play)),
            )
            // Play mode: hide examine cursor
            .add_systems(
                Update,
                examine::hide_examine_cursor.run_if(in_state(GameMode::Play)),
            )
            // Examine mode systems
            .add_systems(
                Update,
                (examine::examine_input_system, examine::examine_cursor_sync)
                    .chain()
                    .run_if(in_state(GameMode::Examine)),
            )
            // egui panels (always)
            .add_systems(
                EguiPrimaryContextPass,
                (
                    stat_panel::stat_panel_system,
                    message_panel::message_panel_system,
                    examine::examine_popup_system,
                )
                    .chain(),
            );
    }
}

// --- Examine Mode ---

fn enter_examine_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    player: Res<Player>,
    mut cursor: ResMut<examine::ExamineCursor>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut messages: ResMut<MessageLog>,
) {
    if keyboard.just_pressed(KeyCode::KeyX) {
        cursor.pos = player.pos;
        cursor.showing_popup = false;
        next_state.set(GameMode::Examine);
        messages.add("Examine mode. Move cursor with arrow keys. Enter to inspect. Escape to exit.");
    }
}

fn spawn_examine_cursor(mut commands: Commands) {
    commands.spawn((
        examine::ExamineCursorSprite,
        Sprite {
            color: Color::srgba(1.0, 1.0, 0.0, 0.3),
            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 2.0),
        Visibility::Hidden,
    ));
}

// --- Camera ---

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scale: 0.5,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn welcome_message(mut messages: ResMut<MessageLog>) {
    messages.add("Welcome to the dungeon! Use arrow keys or hjkl to move.");
    messages.add("Walk into monsters to attack them.");
}

// --- Dungeon Rendering ---

#[derive(Component)]
pub struct TerrainSprite;

#[derive(Component)]
pub struct GridPos(pub Coord);

pub fn feature_to_tile(feature: Feature) -> TileId {
    match feature {
        Feature::Wall => TileId::WallBrickDark,
        Feature::Floor => TileId::FloorGreyDirt,
        Feature::ClosedDoor => TileId::DoorClosed,
        Feature::OpenDoor => TileId::DoorOpen,
        Feature::StairsDown => TileId::StairsDown,
    }
}

fn spawn_dungeon(
    mut commands: Commands,
    terrain_grid: Res<TerrainGrid>,
    tiles: Res<TileRegistry>,
    mut sprite_grid: ResMut<TerrainSpriteGrid>,
) {
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let pos = Coord::new(x as i32, y as i32);
            let tile_id = feature_to_tile(terrain_grid.cells[y][x]);
            let entity = commands
                .spawn((
                    TerrainSprite,
                    GridPos(pos),
                    Sprite::from_image(tiles.get(tile_id)),
                    Transform::from_xyz(x as f32 * TILE_SIZE, -(y as f32) * TILE_SIZE, 0.0),
                ))
                .id();
            sprite_grid.set(pos, Some(entity));
        }
    }
}

// --- Player ---

fn spawn_player(mut commands: Commands, player: Res<Player>, tiles: Res<TileRegistry>) {
    let world = player.pos.to_world();
    commands.spawn((
        PlayerSprite,
        Sprite::from_image(tiles.get(TileId::PlayerHuman)),
        Transform::from_xyz(world.x, world.y, 1.0),
    ));
}

pub fn sync_player_sprite(
    player: Res<Player>,
    mut query: Query<&mut Transform, With<PlayerSprite>>,
) {
    for mut transform in &mut query {
        let world = player.pos.to_world();
        transform.translation.x = world.x;
        transform.translation.y = world.y;
    }
}

pub fn camera_follow(
    player: Res<Player>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<PlayerSprite>)>,
) {
    for mut transform in &mut camera {
        let target = player.pos.to_world();
        transform.translation.x = target.x;
        transform.translation.y = target.y;
    }
}

// --- Input ---

fn player_input(keyboard: Res<ButtonInput<KeyCode>>, mut pending: ResMut<PendingMove>) {
    let delta = if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyH)
    {
        Some((-1, 0))
    } else if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyL) {
        Some((1, 0))
    } else if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyK) {
        Some((0, -1))
    } else if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyJ) {
        Some((0, 1))
    } else if keyboard.just_pressed(KeyCode::KeyY) {
        Some((-1, -1))
    } else if keyboard.just_pressed(KeyCode::KeyU) {
        Some((1, -1))
    } else if keyboard.just_pressed(KeyCode::KeyB) {
        Some((-1, 1))
    } else if keyboard.just_pressed(KeyCode::KeyN) {
        Some((1, 1))
    } else {
        None
    };

    if delta.is_some() {
        pending.command = delta;
    }
}

// --- Action Execution (with door opening) ---

fn execute_player_action(
    mut pending: ResMut<PendingMove>,
    mut player: ResMut<Player>,
    mut terrain: ResMut<TerrainGrid>,
    monster_grid: Res<MonsterGrid>,
    sprite_grid: Res<TerrainSpriteGrid>,
    tiles: Res<TileRegistry>,
    mut monsters: Query<(&MonsterName, &mut Health, &ArmorClass, &Evasion)>,
    mut sprites: Query<&mut Sprite, With<TerrainSprite>>,
    mut messages: ResMut<MessageLog>,
) {
    let Some((dx, dy)) = pending.command.take() else {
        return;
    };

    let target = Coord::new(player.pos.x + dx, player.pos.y + dy);
    if !target.in_bounds() {
        return;
    }

    // Attack monster at target
    if let Some(entity) = monster_grid.get(target) {
        if let Ok((name, mut hp, ac, ev)) = monsters.get_mut(entity) {
            let base_damage = 5 + player.str_stat / 3;
            let accuracy = 10 + player.xl * 2;
            let result = combat::resolve_melee(base_damage, accuracy, ac.0, ev.0);
            if result.hit {
                hp.current -= result.damage;
                if result.damage > 0 {
                    messages.add(format!("You hit the {}! ({} damage)", name.0, result.damage));
                } else {
                    messages.add(format!("You hit the {} but do no damage.", name.0));
                }
            } else {
                messages.add(format!("You miss the {}.", name.0));
            }
            player.turn_is_over = true;
            player.time_taken = 10;
        }
    } else if terrain.get(target) == Some(Feature::ClosedDoor) {
        // Open the door
        terrain.set(target, Feature::OpenDoor);
        if let Some(sprite_entity) = sprite_grid.get(target) {
            if let Ok(mut sprite) = sprites.get_mut(sprite_entity) {
                sprite.image = tiles.get(TileId::DoorOpen);
            }
        }
        messages.add("You open the door.");
        player.turn_is_over = true;
        player.time_taken = 10;
    } else if terrain.is_passable(target) {
        player.pos = target;
        player.turn_is_over = true;
        player.time_taken = 10;
    }
}

// --- Monster Death ---

fn check_monster_death(
    mut commands: Commands,
    query: Query<(Entity, &MonsterName, &Health, &Position), With<MonsterTag>>,
    mut grid: ResMut<MonsterGrid>,
    mut messages: ResMut<MessageLog>,
) {
    for (entity, name, health, pos) in &query {
        if health.current <= 0 {
            messages.add(format!("The {} dies!", name.0));
            grid.set(pos.0, None);
            commands.entity(entity).despawn();
        }
    }
}

// --- Monster Retaliation ---

fn monster_retaliation(
    mut player: ResMut<Player>,
    monsters: Query<(&MonsterName, &MeleeAttack, &HitDice, &Position), With<MonsterTag>>,
    mut messages: ResMut<MessageLog>,
) {
    if !player.turn_is_over {
        return;
    }

    for (name, attack, hd, pos) in &monsters {
        if pos.0.adjacent_to(player.pos) {
            let accuracy = hd.0 * 3 + 5;
            let result = combat::resolve_melee(attack.damage, accuracy, player.ac, player.ev);
            if result.hit && result.damage > 0 {
                player.hp -= result.damage;
                messages.add(format!("The {} hits you! ({} damage)", name.0, result.damage));
            } else if result.hit {
                messages.add(format!("The {} hits you but does no damage.", name.0));
            } else {
                messages.add(format!("The {} misses you.", name.0));
            }
        }
    }

    player.turn_is_over = false;
}

// --- Player Death ---

fn check_player_death(
    player: Res<Player>,
    mut messages: ResMut<MessageLog>,
    mut next_state: ResMut<NextState<GameMode>>,
) {
    if player.hp <= 0 {
        messages.add("You die...");
        messages.add("Press Escape to quit.");
        next_state.set(GameMode::GameOver);
    }
}

// --- Monster Loading & Spawning ---

fn load_monster_defs(mut defs: ResMut<MonsterDefs>) {
    let base_path = "crawl-ref/source/dat/mons";
    let names = ["goblin", "kobold", "rat", "bat", "jackal", "gnoll", "orc"];

    for name in &names {
        let path = format!("{}/{}.yaml", base_path, name);
        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_yaml::from_str::<MonsterDef>(&content) {
                Ok(def) => {
                    defs.0.insert(name.to_string(), def);
                }
                Err(e) => eprintln!("Failed to parse {}: {}", path, e),
            },
            Err(e) => eprintln!("Failed to read {}: {}", path, e),
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
        let Some(def) = defs.0.get(*name) else {
            eprintln!("Monster def not found: {}", name);
            continue;
        };
        let Some(tile_id) = dcss_tiles::monster_name_to_tile(name) else {
            eprintln!("No tile for monster: {}", name);
            continue;
        };

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
