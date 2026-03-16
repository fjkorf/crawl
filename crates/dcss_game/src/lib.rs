//! DcssGamePlugin — shared game setup used by the main binary and test examples.
//! Now includes monster AI (chase + attack) and random monster placement.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use rand::Rng;

use dcss_core::combat;
use dcss_core::message::MessageLog;
use dcss_core::monster::*;
use dcss_core::player::{Player, PlayerSprite};
use dcss_core::terrain::{self, Feature, TerrainGrid, TerrainSpriteGrid};
use dcss_core::turn::{GameMode, PendingMove};
use dcss_core::types::*;
use dcss_lua::des_parser;
use dcss_lua::lua_state;
use dcss_tiles::{self, TileId, TileRegistry, TILE_SIZE};
use dcss_ui::{examine, message_panel, stat_panel};

/// Controls where the dungeon comes from.
#[derive(Resource, Default)]
pub enum DungeonSource {
    #[default]
    Hardcoded,
    DesVault { des_file: String, vault_index: usize },
}

pub struct DcssGamePlugin;

impl Plugin for DcssGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameMode>()
            .insert_resource(ClearColor(Color::srgb(0.1, 0.0, 0.2)))
            .init_resource::<DungeonSource>()
            .init_resource::<Player>()
            .init_resource::<MonsterGrid>()
            .init_resource::<MonsterDefs>()
            .init_resource::<MessageLog>()
            .init_resource::<PendingMove>()
            .init_resource::<TerrainSpriteGrid>()
            .init_resource::<examine::ExamineCursor>()
            .init_resource::<examine::MonsterInfoState>()
            .add_systems(Startup, (dcss_tiles::load_tiles, setup_camera))
            .add_systems(Startup,
                (load_monster_defs, generate_dungeon, spawn_dungeon, spawn_player, spawn_monsters, spawn_examine_cursor, welcome_message)
                    .chain().after(dcss_tiles::load_tiles))
            .add_systems(Update, player_input.run_if(in_state(GameMode::Play)))
            .add_systems(Update,
                (execute_player_action, check_monster_death, monster_ai, check_player_death)
                    .chain().run_if(in_state(GameMode::Play)).after(player_input))
            .add_systems(Update,
                (sync_player_sprite, sync_monster_sprites, camera_follow)
                    .run_if(in_state(GameMode::Play)).after(execute_player_action))
            .add_systems(Update,
                (enter_examine_mode, examine::hide_examine_cursor).run_if(in_state(GameMode::Play)))
            .add_systems(Update,
                (examine::examine_input_system, examine::examine_cursor_sync)
                    .chain().run_if(in_state(GameMode::Examine)))
            .add_systems(EguiPrimaryContextPass,
                (stat_panel::stat_panel_system, message_panel::message_panel_system, examine::examine_popup_system).chain());
    }
}

fn enter_examine_mode(keyboard: Res<ButtonInput<KeyCode>>, player: Res<Player>,
    mut cursor: ResMut<examine::ExamineCursor>, mut next_state: ResMut<NextState<GameMode>>, mut messages: ResMut<MessageLog>) {
    if keyboard.just_pressed(KeyCode::KeyX) {
        cursor.pos = player.pos;
        cursor.showing_popup = false;
        next_state.set(GameMode::Examine);
        messages.add("Examine mode. Arrow keys to move. Enter to inspect. Escape to exit.");
    }
}

fn spawn_examine_cursor(mut commands: Commands) {
    commands.spawn((examine::ExamineCursorSprite,
        Sprite { color: Color::srgba(1.0, 1.0, 0.0, 0.3), custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)), ..default() },
        Transform::from_xyz(0.0, 0.0, 2.0), Visibility::Hidden));
}

fn generate_dungeon(mut commands: Commands, source: Res<DungeonSource>, mut player: ResMut<Player>, mut messages: ResMut<MessageLog>) {
    let (grid, pos) = match &*source {
        DungeonSource::Hardcoded => (terrain::hardcoded_dungeon(), Coord::new(5, 5)),
        DungeonSource::DesVault { des_file, vault_index } => {
            match load_des_vault(des_file, *vault_index) {
                Ok((g, p, name)) => { messages.add(format!("Loaded vault: {}", name)); (g, p) }
                Err(e) => { eprintln!("Vault load failed: {}", e); (terrain::hardcoded_dungeon(), Coord::new(5, 5)) }
            }
        }
    };
    player.pos = pos;
    commands.insert_resource(grid);
}

fn load_des_vault(des_file: &str, vault_index: usize) -> Result<(TerrainGrid, Coord, String), String> {
    let content = std::fs::read_to_string(des_file).map_err(|e| format!("{}: {}", des_file, e))?;
    let parsed = des_parser::parse_des_file(&content, des_file)?;
    if vault_index >= parsed.maps.len() { return Err(format!("index {} out of range", vault_index)) }
    let lua = lua_state::create_dlua_with_base("crawl-ref/source/dat/dlua").map_err(|e| format!("lua: {}", e))?;
    if !parsed.global_prelude.is_empty() { let _ = lua.load(&parsed.global_prelude).exec(); }
    let map_def = lua_state::execute_raw_map(&lua, &parsed.maps[vault_index]).map_err(|e| format!("{}", e))?;
    let (grid, pos) = terrain::from_map_lines(&map_def.map_lines);
    Ok((grid, pos, map_def.name))
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Projection::from(OrthographicProjection { scale: 0.5, ..OrthographicProjection::default_2d() })));
}

fn welcome_message(mut messages: ResMut<MessageLog>) {
    messages.add("Welcome to the dungeon! Use arrow keys or hjkl to move.");
    messages.add("Walk into monsters to attack. Press x to examine.");
}

#[derive(Component)] pub struct TerrainSpriteMarker;
#[derive(Component)] pub struct GridPos(pub Coord);

fn feature_to_tile(f: Feature) -> TileId {
    match f { Feature::Wall => TileId::WallBrickDark, Feature::Floor => TileId::FloorGreyDirt,
              Feature::ClosedDoor => TileId::DoorClosed, Feature::OpenDoor => TileId::DoorOpen,
              Feature::StairsDown => TileId::StairsDown }
}

fn spawn_dungeon(mut commands: Commands, grid: Res<TerrainGrid>, tiles: Res<TileRegistry>, mut sg: ResMut<TerrainSpriteGrid>) {
    for y in 0..MAP_HEIGHT { for x in 0..MAP_WIDTH {
        let pos = Coord::new(x as i32, y as i32);
        let e = commands.spawn((TerrainSpriteMarker, GridPos(pos),
            Sprite::from_image(tiles.get(feature_to_tile(grid.cells[y][x]))),
            Transform::from_xyz(x as f32 * TILE_SIZE, -(y as f32) * TILE_SIZE, 0.0))).id();
        sg.set(pos, Some(e));
    }}
}

fn spawn_player(mut commands: Commands, player: Res<Player>, tiles: Res<TileRegistry>) {
    let w = player.pos.to_world();
    commands.spawn((PlayerSprite, Sprite::from_image(tiles.get(TileId::PlayerHuman)),
        Transform::from_xyz(w.x, w.y, 1.0)));
}

fn sync_player_sprite(player: Res<Player>, mut q: Query<&mut Transform, With<PlayerSprite>>) {
    for mut t in &mut q { let w = player.pos.to_world(); t.translation.x = w.x; t.translation.y = w.y; }
}

fn sync_monster_sprites(mut q: Query<(&Position, &mut Transform), (With<MonsterTag>, Without<PlayerSprite>, Without<Camera2d>)>) {
    for (pos, mut t) in &mut q { let w = pos.0.to_world(); t.translation.x = w.x; t.translation.y = w.y; }
}

fn camera_follow(player: Res<Player>, mut q: Query<&mut Transform, (With<Camera2d>, Without<PlayerSprite>, Without<MonsterTag>)>) {
    for mut t in &mut q { let w = player.pos.to_world(); t.translation.x = w.x; t.translation.y = w.y; }
}

fn player_input(keyboard: Res<ButtonInput<KeyCode>>, mut pending: ResMut<PendingMove>) {
    let d = if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyH) { Some((-1,0)) }
    else if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyL) { Some((1,0)) }
    else if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyK) { Some((0,-1)) }
    else if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyJ) { Some((0,1)) }
    else if keyboard.just_pressed(KeyCode::KeyY) { Some((-1,-1)) }
    else if keyboard.just_pressed(KeyCode::KeyU) { Some((1,-1)) }
    else if keyboard.just_pressed(KeyCode::KeyB) { Some((-1,1)) }
    else if keyboard.just_pressed(KeyCode::KeyN) { Some((1,1)) }
    else { None };
    if d.is_some() { pending.command = d; }
}

fn execute_player_action(mut pending: ResMut<PendingMove>, mut player: ResMut<Player>,
    mut terrain: ResMut<TerrainGrid>, mg: Res<MonsterGrid>, sg: Res<TerrainSpriteGrid>,
    tiles: Res<TileRegistry>, mut monsters: Query<(&MonsterName, &mut Health, &ArmorClass, &Evasion)>,
    mut sprites: Query<&mut Sprite, With<TerrainSpriteMarker>>, mut messages: ResMut<MessageLog>) {
    let Some((dx, dy)) = pending.command.take() else { return };
    let target = Coord::new(player.pos.x + dx, player.pos.y + dy);
    if !target.in_bounds() { return }
    if let Some(entity) = mg.get(target) {
        if let Ok((name, mut hp, ac, ev)) = monsters.get_mut(entity) {
            let result = combat::resolve_melee(5 + player.str_stat / 3, 10 + player.xl * 2, ac.0, ev.0);
            if result.hit && result.damage > 0 { hp.current -= result.damage; messages.add(format!("You hit the {}! ({} damage)", name.0, result.damage)); }
            else if result.hit { messages.add(format!("You hit the {} but do no damage.", name.0)); }
            else { messages.add(format!("You miss the {}.", name.0)); }
            player.turn_is_over = true; player.time_taken = 10;
        }
    } else if terrain.get(target) == Some(Feature::ClosedDoor) {
        terrain.set(target, Feature::OpenDoor);
        if let Some(se) = sg.get(target) { if let Ok(mut s) = sprites.get_mut(se) { s.image = tiles.get(TileId::DoorOpen); } }
        messages.add("You open the door."); player.turn_is_over = true; player.time_taken = 10;
    } else if terrain.is_passable(target) {
        player.pos = target; player.turn_is_over = true; player.time_taken = 10;
    }
}

fn check_monster_death(mut commands: Commands, q: Query<(Entity, &MonsterName, &Health, &Position), With<MonsterTag>>,
    mut grid: ResMut<MonsterGrid>, mut messages: ResMut<MessageLog>) {
    for (entity, name, health, pos) in &q {
        if health.current <= 0 { messages.add(format!("The {} dies!", name.0)); grid.set(pos.0, None); commands.entity(entity).despawn(); }
    }
}

/// Monster AI: adjacent monsters attack, nearby monsters chase.
fn monster_ai(mut player: ResMut<Player>,
    mut monsters: Query<(Entity, &MonsterName, &MeleeAttack, &HitDice, &mut Position), With<MonsterTag>>,
    terrain: Res<TerrainGrid>, mut grid: ResMut<MonsterGrid>, mut messages: ResMut<MessageLog>) {
    if !player.turn_is_over { return }
    let pp = player.pos;

    // Collect actions first to avoid borrow conflicts
    let mut attacks = Vec::new();
    let mut moves = Vec::new();

    for (entity, name, attack, hd, pos) in monsters.iter() {
        let dist = (pos.0.x - pp.x).abs().max((pos.0.y - pp.y).abs());
        if dist <= 1 {
            attacks.push((name.0.clone(), attack.damage, attack.attack_type.clone(), hd.0));
        } else if dist <= 8 {
            let dx = (pp.x - pos.0.x).signum();
            let dy = (pp.y - pos.0.y).signum();
            for (mx, my) in [(dx, dy), (dx, 0), (0, dy)] {
                if mx == 0 && my == 0 { continue }
                let np = Coord::new(pos.0.x + mx, pos.0.y + my);
                if np == pp { break }
                if terrain.is_passable(np) && grid.get(np).is_none() {
                    moves.push((entity, pos.0, np));
                    break;
                }
            }
        }
    }

    // Apply attacks
    for (name, dmg, _atype, hd) in &attacks {
        let result = combat::resolve_melee(*dmg, hd * 3 + 5, player.ac, player.ev);
        if result.hit && result.damage > 0 { player.hp -= result.damage; messages.add(format!("The {} hits you! ({} damage)", name, result.damage)); }
        else if result.hit { messages.add(format!("The {} hits you but does no damage.", name)); }
        else { messages.add(format!("The {} misses you.", name)); }
    }

    // Apply moves
    for (entity, old_pos, new_pos) in &moves {
        grid.set(*old_pos, None);
        grid.set(*new_pos, Some(*entity));
        if let Ok((_, _, _, _, mut pos)) = monsters.get_mut(*entity) {
            pos.0 = *new_pos;
        }
    }

    player.turn_is_over = false;
}

fn check_player_death(player: Res<Player>, mut messages: ResMut<MessageLog>, mut ns: ResMut<NextState<GameMode>>) {
    if player.hp <= 0 { messages.add("You die..."); messages.add("Press Escape to quit."); ns.set(GameMode::GameOver); }
}

fn load_monster_defs(mut defs: ResMut<MonsterDefs>) {
    let base = "crawl-ref/source/dat/mons";
    for name in ["goblin", "kobold", "rat", "bat", "jackal", "gnoll", "orc"] {
        if let Ok(c) = std::fs::read_to_string(format!("{}/{}.yaml", base, name)) {
            if let Ok(def) = serde_yaml::from_str::<MonsterDef>(&c) { defs.0.insert(name.into(), def); }
        }
    }
}

fn spawn_monsters(mut commands: Commands, defs: Res<MonsterDefs>, tiles: Res<TileRegistry>,
    terrain: Res<TerrainGrid>, player: Res<Player>, mut grid: ResMut<MonsterGrid>) {
    let mut rng = rand::rng();
    let mut floor_tiles = Vec::new();
    for y in 0..MAP_HEIGHT { for x in 0..MAP_WIDTH {
        let p = Coord::new(x as i32, y as i32);
        if terrain.cells[y][x] == Feature::Floor && grid.get(p).is_none() && p != player.pos {
            floor_tiles.push(p);
        }
    }}

    let names = ["goblin", "rat", "kobold", "gnoll", "orc", "jackal", "bat"];
    let count = 7.min(floor_tiles.len());
    let mut positions = floor_tiles;
    for i in 0..count {
        if positions.is_empty() { break }
        let idx = rng.random_range(0..positions.len());
        let pos = positions.swap_remove(idx);
        let name = names[i % names.len()];
        let Some(def) = defs.0.get(name) else { continue };
        let Some(tid) = dcss_tiles::monster_name_to_tile(name) else { continue };
        let hp = def.hp_10x / 10;
        let atk = def.attacks.first().map(|a| MeleeAttack { attack_type: a.attack_type.clone(), damage: a.damage })
            .unwrap_or(MeleeAttack { attack_type: "hit".into(), damage: 1 });
        let w = pos.to_world();
        let e = commands.spawn((MonsterTag, MonsterName(def.name.clone()),
            Health { current: hp, max: hp }, Position(pos),
            Speed { base: def.speed.unwrap_or(10), energy: 0 },
            atk, ArmorClass(def.ac), Evasion(def.ev), HitDice(def.hd),
            Sprite::from_image(tiles.get(tid)), Transform::from_xyz(w.x, w.y, 1.0))).id();
        grid.set(pos, Some(e));
    }
}
