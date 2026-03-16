//! DcssGamePlugin — shared game setup used by the main binary and test examples.
//! Now includes monster AI (chase + attack) and random monster placement.

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use rand::Rng;

use dcss_core::chargen::{self, ChargenState, SpeciesDefs, JobDefs};
use dcss_core::combat;
use dcss_core::fov::VisibilityMap;
use dcss_core::pathfind;
use dcss_core::item::{self, Inventory, ItemTag, ItemName, ItemData, ItemPosition};
use dcss_core::level::{CurrentLevel, LevelStore, SavedLevel, SavedMonster, SavedItem, StairsAction, StairsDirection, MAX_DEPTH};
use dcss_core::message::MessageLog;
use dcss_core::monster::*;
use dcss_core::player::{Player, PlayerSprite};
use dcss_core::terrain::{self, Feature, TerrainGrid, TerrainSpriteGrid};
use dcss_core::turn::{GameMode, GamePhase, PendingMove};
use dcss_core::types::*;
use dcss_lua::des_parser;
use dcss_lua::lua_state;
use dcss_tiles::{self, TileId, TileRegistry, TILE_SIZE};
use dcss_ui::{examine, LituiState, render_chargen, render_inventory, render_stat_panel, render_message_log, render_death_screen};

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
        app.init_state::<GamePhase>()
            .init_state::<GameMode>()
            .insert_resource(ClearColor(Color::srgb(0.1, 0.0, 0.2)))
            .init_resource::<DungeonSource>()
            .init_resource::<Player>()
            .init_resource::<MonsterGrid>()
            .init_resource::<MonsterDefs>()
            .init_resource::<MessageLog>()
            .init_resource::<PendingMove>()
            .init_resource::<TerrainSpriteGrid>()
            .init_resource::<examine::ExamineCursor>()
            .init_resource::<LituiState>()
            .init_resource::<CurrentLevel>()
            .init_resource::<StairsAction>()
            .init_resource::<LevelStore>()
            .init_resource::<VisibilityMap>()
            .init_resource::<SpeciesDefs>()
            .init_resource::<JobDefs>()
            .init_resource::<ChargenState>()
            .init_resource::<Inventory>()
            // Startup: load data + camera (always needed)
            .add_systems(Startup, (dcss_tiles::load_tiles, setup_camera, load_chargen_data_system, load_monster_defs))
            // Character creation phase
            .add_systems(Update,
                (populate_chargen_state, check_chargen_complete)
                    .chain().run_if(in_state(GamePhase::CharacterCreation)))
            .add_systems(EguiPrimaryContextPass,
                render_chargen_screen.run_if(in_state(GamePhase::CharacterCreation)))
            // Playing phase: setup when entering
            .add_systems(OnEnter(GamePhase::Playing),
                (generate_dungeon, spawn_dungeon, spawn_player, spawn_monsters, spawn_floor_items, spawn_examine_cursor, welcome_message)
                    .chain())
            // Playing phase: gameplay
            .add_systems(Update, autoexplore_system.run_if(in_state(GameMode::Play)).run_if(in_state(GamePhase::Playing)))
            .add_systems(Update, player_input.run_if(in_state(GameMode::Play)).run_if(in_state(GamePhase::Playing)).after(autoexplore_system))
            .add_systems(Update,
                (execute_player_action, handle_stairs_input, check_monster_death, monster_ai, check_player_death)
                    .chain().run_if(in_state(GameMode::Play)).run_if(in_state(GamePhase::Playing)).after(player_input))
            .add_systems(Update,
                (update_fov, sync_player_sprite, sync_monster_sprites, apply_visibility, camera_follow)
                    .chain().run_if(in_state(GamePhase::Playing)).after(execute_player_action))
            .add_systems(Update,
                (enter_examine_mode, toggle_inventory, handle_item_use, examine::hide_examine_cursor)
                    .run_if(in_state(GameMode::Play)).run_if(in_state(GamePhase::Playing)))
            .add_systems(Update,
                (examine::examine_input_system, examine::examine_cursor_sync)
                    .chain().run_if(in_state(GameMode::Examine)))
            .add_systems(Update, close_inventory.run_if(in_state(GameMode::Inventory)))
            .add_systems(Update, populate_inventory_state.run_if(in_state(GameMode::Inventory)))
            // egui panels (during playing)
            .add_systems(EguiPrimaryContextPass,
                (render_stat_panel_system, render_message_log_system,
                 examine::examine_popup_system)
                    .chain().run_if(in_state(GamePhase::Playing)))
            .add_systems(EguiPrimaryContextPass,
                render_inventory_screen.run_if(in_state(GameMode::Inventory)))
            // Death/win screen
            .add_systems(EguiPrimaryContextPass,
                render_death_screen_system.run_if(in_state(GameMode::GameOver).or(in_state(GameMode::Won))))
            .add_systems(Update,
                death_screen_input.run_if(in_state(GameMode::GameOver).or(in_state(GameMode::Won))));
    }
}

// --- Character Creation (litui) ---

fn load_chargen_data_system(mut species: ResMut<SpeciesDefs>, mut jobs: ResMut<JobDefs>) {
    chargen::load_chargen_data(&mut species, &mut jobs);
}

fn populate_chargen_state(
    species: Res<SpeciesDefs>,
    jobs: Res<JobDefs>,
    mut state: ResMut<LituiState>,
) {
    // Populate species list if empty
    if state.species_list.is_empty() {
        state.species_list = species.0.iter().map(|s| format!("{} ({})", s.name, s.difficulty)).collect();
    }
    if state.job_list.is_empty() {
        state.job_list = jobs.0.iter().map(|j| format!("{} ({})", j.name, j.category)).collect();
    }

    // Update preview from current selection
    if let (Some(sp), Some(job)) = (species.0.get(state.chosen_species), jobs.0.get(state.chosen_job)) {
        state.preview_name = format!("{} {}", sp.name, job.name);
        state.preview_str = format!("{}", sp.str_stat());
        state.preview_int = format!("{}", sp.int_stat());
        state.preview_dex = format!("{}", sp.dex_stat());
        let skills: Vec<String> = job.skills.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
        state.preview_skills = if skills.is_empty() { "None".into() } else { skills.join(", ") };
    }
}

// --- Stat Panel (litui) ---

fn render_stat_panel_system(mut contexts: EguiContexts, mut state: ResMut<LituiState>, player: Res<Player>) -> Result {
    // Populate stat panel fields
    let hp_frac = player.hp as f64 / player.max_hp.max(1) as f64;
    state.hp_text = format!("{}/{}", player.hp, player.max_hp);
    state.hp_frac = hp_frac;
    state.hp_style = if hp_frac > 0.5 { "hp_good".into() } else if hp_frac > 0.25 { "hp_warn".into() } else { "hp_danger".into() };
    state.mp_text = format!("{}/{}", player.mp, player.max_mp);
    state.mp_frac = player.mp as f64 / player.max_mp.max(1) as f64;
    state.mp_style = "mp_style".into();
    state.stat_ac = format!("{}", player.total_ac());
    state.stat_ev = format!("{}", player.ev);
    state.stat_str = format!("{}", player.str_stat);
    state.stat_int = format!("{}", player.int_stat);
    state.stat_dex = format!("{}", player.dex_stat);
    state.stat_xl = format!("{}", player.xl);
    state.stat_xp = format!("{}/{}", player.xp, player.xp_next);
    state.stat_gold = format!("{}", player.gold);
    state.stat_weapon = player.weapon_name();
    state.stat_armour = player.armour_name();
    let mut status_parts = Vec::new();
    if player.status.poison > 0 { status_parts.push(format!("Poison({})", player.status.poison)); }
    if player.status.might > 0 { status_parts.push(format!("Might({})", player.status.might)); }
    if player.status.haste > 0 { status_parts.push(format!("Haste({})", player.status.haste)); }
    state.stat_status = if status_parts.is_empty() { String::new() } else { status_parts.join(" ") };
    state.has_orb = player.has_orb;

    egui::SidePanel::right("stats").resizable(false).exact_width(180.0)
        .show(contexts.ctx_mut()?, |ui| { render_stat_panel(ui, &mut state); });
    Ok(())
}

// --- Message Log (litui) ---

fn render_message_log_system(mut contexts: EguiContexts, mut state: ResMut<LituiState>, msg_log: Res<MessageLog>) -> Result {
    state.messages = msg_log.messages.clone();
    egui::TopBottomPanel::bottom("messages").resizable(true).default_height(120.0)
        .show(contexts.ctx_mut()?, |ui| { render_message_log(ui, &mut state); });
    Ok(())
}

// --- Character Creation (litui) ---

fn render_chargen_screen(mut contexts: EguiContexts, mut state: ResMut<LituiState>) -> Result {
    egui::CentralPanel::default().show(contexts.ctx_mut()?, |ui| {
        render_chargen(ui, &mut state);
    });
    Ok(())
}

fn check_chargen_complete(
    mut litui_state: ResMut<LituiState>,
    species: Res<SpeciesDefs>,
    jobs: Res<JobDefs>,
    mut player: ResMut<Player>,
    mut messages: ResMut<MessageLog>,
    mut next_phase: ResMut<NextState<GamePhase>>,
) {
    if litui_state.start_game_count == 0 { return }
    litui_state.start_game_count = 0;

    let species_idx = litui_state.chosen_species;
    let job_idx = litui_state.chosen_job;
    if let (Some(sp), Some(job)) = (species.0.get(species_idx), jobs.0.get(job_idx)) {
        player.str_stat = sp.str_stat();
        player.int_stat = sp.int_stat();
        player.dex_stat = sp.dex_stat();
        player.hp = 15 + sp.str_stat();
        player.max_hp = player.hp;
        player.mp = 3 + sp.int_stat() / 2;
        player.max_mp = player.mp;
        player.species_name = sp.name.clone();
        player.job_name = job.name.clone();
        messages.add(format!("You are a {} {}.", sp.name, job.name));
    }
    next_phase.set(GamePhase::Playing);
}

// --- Inventory ---

fn toggle_inventory(keyboard: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameMode>>) {
    if keyboard.just_pressed(KeyCode::KeyI) {
        next_state.set(GameMode::Inventory);
    }
}

fn populate_inventory_state(inventory: Res<Inventory>, mut state: ResMut<LituiState>) {
    state.gold_amount = format!("{}", inventory.gold);
    state.items.clear();
    for (i, item) in inventory.items.iter().enumerate() {
        let mut row = dcss_ui::pages::ItemsRow::default();
        row.letter = format!("{}", (b'a' + i as u8) as char);
        row.name = item.name.clone();
        row.qty = if item.quantity > 1 { format!("x{}", item.quantity) } else { String::new() };
        state.items.push(row);
    }
}

fn render_inventory_screen(mut contexts: EguiContexts, mut state: ResMut<LituiState>) -> Result {
    egui::Window::new("Inventory")
        .collapsible(false)
        .resizable(true)
        .default_width(350.0)
        .show(contexts.ctx_mut()?, |ui| {
            render_inventory(ui, &mut state);
        });
    Ok(())
}

fn handle_item_use(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: ResMut<Player>,
    mut inventory: ResMut<Inventory>,
    mut messages: ResMut<MessageLog>,
) {
    // q = quaff first potion
    if keyboard.just_pressed(KeyCode::KeyQ) {
        if let Some(idx) = inventory.items.iter().position(|i| i.class == item::ItemClass::Potion) {
            let potion = inventory.items.remove(idx);
            match potion.subtype.as_str() {
                "healing" => {
                    let healed = (player.max_hp - player.hp).min(10 + player.xl * 2);
                    player.hp += healed;
                    messages.add(format!("You drink the {}. (+{} HP)", potion.name, healed));
                }
                "might" => {
                    player.status.might = 20;
                    messages.add(format!("You drink the {}. You feel mighty!", potion.name));
                }
                "haste" => {
                    player.status.haste = 15;
                    messages.add(format!("You drink the {}. You feel fast!", potion.name));
                }
                _ => messages.add(format!("You drink the {}.", potion.name)),
            }
            player.turn_is_over = true;
            player.time_taken = 10;
        } else {
            messages.add("You have no potions.");
        }
    }

    // r = read first scroll
    if keyboard.just_pressed(KeyCode::KeyR) {
        if let Some(idx) = inventory.items.iter().position(|i| i.class == item::ItemClass::Scroll) {
            let scroll = inventory.items.remove(idx);
            match scroll.subtype.as_str() {
                "teleportation" => {
                    // Teleport to random floor tile (handled in dcss_game by setting a flag)
                    messages.add("You read the scroll of teleportation. You are teleported!");
                    // Find random floor tile
                    // (simplified: just move player randomly — proper implementation needs terrain access)
                    let mut rng = rand::rng();
                    let dx = rng.random_range(-5..=5);
                    let dy = rng.random_range(-5..=5);
                    player.pos = Coord::new(player.pos.x + dx, player.pos.y + dy);
                }
                "blinking" => {
                    messages.add("You read the scroll of blinking. You blink!");
                    let mut rng = rand::rng();
                    let dx = rng.random_range(-3..=3);
                    let dy = rng.random_range(-3..=3);
                    player.pos = Coord::new(player.pos.x + dx, player.pos.y + dy);
                }
                _ => messages.add(format!("You read the {}.", scroll.name)),
            }
            player.turn_is_over = true;
            player.time_taken = 10;
        } else {
            messages.add("You have no scrolls.");
        }
    }

    // w = wield first weapon
    if keyboard.just_pressed(KeyCode::KeyW) && !keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        if let Some(idx) = inventory.items.iter().position(|i| i.class == item::ItemClass::Weapon) {
            let weapon = inventory.items.remove(idx);
            messages.add(format!("You wield the {}.", weapon.name));
            if let Some(old) = player.equipped_weapon.replace(weapon) {
                inventory.items.push(old);
            }
        } else {
            messages.add("You have no weapons.");
        }
    }

    // W (shift+w) = wear first armour
    if keyboard.just_pressed(KeyCode::KeyW) && keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        if let Some(idx) = inventory.items.iter().position(|i| i.class == item::ItemClass::Armour) {
            let armour = inventory.items.remove(idx);
            messages.add(format!("You wear the {}.", armour.name));
            if let Some(old) = player.equipped_armour.replace(armour) {
                inventory.items.push(old);
            }
        } else {
            messages.add("You have no armour.");
        }
    }
}

fn close_inventory(keyboard: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameMode>>) {
    if keyboard.just_pressed(KeyCode::KeyI) || keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameMode::Play);
    }
}

// --- Floor Items ---

fn spawn_floor_items(
    mut commands: Commands,
    terrain: Res<TerrainGrid>,
    player: Res<Player>,
    level: Res<CurrentLevel>,
    grid: Res<MonsterGrid>,
    tiles: Res<TileRegistry>,
) {
    spawn_items_for_level(&mut commands, &terrain, &player, &grid, level.depth, &tiles);
}

// --- Examine Mode ---

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
    let mut grid = grid;
    terrain::ensure_stairs(&mut grid, 1, MAX_DEPTH);
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
    messages.add("Welcome to the dungeon!");
    messages.add("Move: arrow keys or hjkl. Attack: walk into monsters.");
    messages.add("Stairs: Shift+> down, Shift+< up. Examine: x. Inventory: i.");
    messages.add("Quaff potion: q. Read scroll: r. Wield weapon: w. Wear armour: Shift+W.");
    messages.add("Find the Orb of Zot on D:5 and return to the surface!");
}

#[derive(Component)] pub struct TerrainSpriteMarker;
#[derive(Component)] pub struct GridPos(pub Coord);

fn feature_to_tile(f: Feature) -> TileId {
    match f {
        Feature::Wall => TileId::WallBrickDark,
        Feature::Floor => TileId::FloorGreyDirt,
        Feature::ClosedDoor => TileId::DoorClosed,
        Feature::OpenDoor => TileId::DoorOpen,
        Feature::StairsDown => TileId::StairsDown,
        Feature::StairsUp => TileId::StairsUp,
    }
}

fn spawn_dungeon(mut commands: Commands, grid: Res<TerrainGrid>, tiles: Res<TileRegistry>, mut sg: ResMut<TerrainSpriteGrid>) {
    for y in 0..MAP_HEIGHT { for x in 0..MAP_WIDTH {
        let pos = Coord::new(x as i32, y as i32);
        let feature = grid.cells[y][x];
        let wx = x as f32 * TILE_SIZE;
        let wy = -(y as f32) * TILE_SIZE;

        // Features with transparency (doors, stairs) need a floor tile underneath
        let needs_floor = matches!(feature,
            Feature::ClosedDoor | Feature::OpenDoor | Feature::StairsDown | Feature::StairsUp);

        if needs_floor {
            commands.spawn((Sprite::from_image(tiles.get(TileId::FloorGreyDirt)),
                Transform::from_xyz(wx, wy, 0.0)));
        }

        let z = if needs_floor { 0.1 } else { 0.0 };
        let e = commands.spawn((TerrainSpriteMarker, GridPos(pos),
            Sprite::from_image(tiles.get(feature_to_tile(feature))),
            Transform::from_xyz(wx, wy, z))).id();
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

fn update_fov(player: Res<Player>, terrain: Res<TerrainGrid>, mut vis: ResMut<VisibilityMap>) {
    vis.calculate(player.pos, 8, &terrain);
}

fn apply_visibility(
    vis: Res<VisibilityMap>,
    mut terrain_sprites: Query<(&GridPos, &mut Sprite), (With<TerrainSpriteMarker>, Without<MonsterTag>)>,
    mut monster_sprites: Query<(&Position, &mut Sprite, &mut Visibility), (With<MonsterTag>, Without<TerrainSpriteMarker>)>,
) {
    // Tint terrain based on visibility
    for (gpos, mut sprite) in &mut terrain_sprites {
        if vis.is_visible(gpos.0) {
            sprite.color = Color::WHITE;
        } else if vis.is_explored(gpos.0) {
            sprite.color = Color::srgba(0.3, 0.3, 0.4, 1.0);
        } else {
            sprite.color = Color::srgba(0.0, 0.0, 0.0, 1.0);
        }
    }

    // Show/hide monsters based on visibility
    for (pos, mut sprite, mut vis_component) in &mut monster_sprites {
        if vis.is_visible(pos.0) {
            sprite.color = Color::WHITE;
            *vis_component = Visibility::Visible;
        } else {
            *vis_component = Visibility::Hidden;
        }
    }
}

fn camera_follow(player: Res<Player>, mut q: Query<&mut Transform, (With<Camera2d>, Without<PlayerSprite>, Without<MonsterTag>)>) {
    for mut t in &mut q { let w = player.pos.to_world(); t.translation.x = w.x; t.translation.y = w.y; }
}

fn autoexplore_system(
    mut player: ResMut<Player>,
    mut pending: ResMut<PendingMove>,
    terrain: Res<TerrainGrid>,
    vis: Res<VisibilityMap>,
    grid: Res<MonsterGrid>,
    keyboard: Res<ButtonInput<KeyCode>>,
    monsters: Query<&Position, With<MonsterTag>>,
    mut messages: ResMut<MessageLog>,
) {
    // Toggle autoexplore on 'o'
    if keyboard.just_pressed(KeyCode::KeyO) {
        player.autoexploring = !player.autoexploring;
        if player.autoexploring {
            messages.add("Autoexploring...");
        } else {
            messages.add("Stopped exploring.");
        }
    }

    // Any other key press stops autoexplore
    if player.autoexploring && keyboard.get_just_pressed().any(|k| *k != KeyCode::KeyO) {
        player.autoexploring = false;
    }

    if !player.autoexploring { return }

    // Interrupt if monster in FOV
    for pos in monsters.iter() {
        if vis.is_visible(pos.0) {
            player.autoexploring = false;
            messages.add("You see a monster! Stopped exploring.");
            return;
        }
    }

    // Find next step toward nearest unexplored tile
    if let Some(next) = pathfind::nearest_unexplored(player.pos, &terrain, &grid, &vis.explored) {
        let dx = next.x - player.pos.x;
        let dy = next.y - player.pos.y;
        pending.command = Some((dx.signum(), dy.signum()));
    } else {
        player.autoexploring = false;
        messages.add("Fully explored.");
    }
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

fn handle_stairs_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: ResMut<Player>,
    terrain: Res<TerrainGrid>,
    mut level: ResMut<CurrentLevel>,
    mut messages: ResMut<MessageLog>,
    mut commands: Commands,
    monster_query: Query<(Entity, &MonsterName, &Health, &Position, &ArmorClass, &Evasion, &HitDice, &Speed, &MeleeAttack), With<MonsterTag>>,
    item_query: Query<(Entity, &ItemName, &ItemData, &ItemPosition), With<ItemTag>>,
    terrain_sprites: Query<Entity, With<TerrainSpriteMarker>>,
    mut grid: ResMut<MonsterGrid>,
    mut sprite_grid: ResMut<TerrainSpriteGrid>,
    mut store: ResMut<LevelStore>,
    mut next_mode: ResMut<NextState<GameMode>>,
    tiles: Res<TileRegistry>,
    defs: Res<MonsterDefs>,
) {
    // Shift+> = descend, Shift+< = ascend
    let going_down = keyboard.just_pressed(KeyCode::Period) && keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let going_up = keyboard.just_pressed(KeyCode::Comma) && keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if !going_down && !going_up { return }

    let on_feature = terrain.get(player.pos);
    if going_down && on_feature != Some(Feature::StairsDown) {
        messages.add("There are no stairs down here.");
        return;
    }
    if going_up && on_feature != Some(Feature::StairsUp) {
        if level.depth == 1 {
            if player.has_orb {
                messages.add("You escape with the Orb of Zot! You win!");
                next_mode.set(GameMode::Won);
                return;
            } else {
                messages.add("You cannot leave without the Orb.");
                return;
            }
        }
        messages.add("There are no stairs up here.");
        return;
    }

    // Save current level
    let saved_monsters: Vec<SavedMonster> = monster_query.iter().map(|(_, name, hp, pos, ac, ev, hd, spd, atk)| {
        SavedMonster {
            name: name.0.clone(), pos: pos.0, hp: hp.current, max_hp: hp.max,
            ac: ac.0, ev: ev.0, hd: hd.0, speed: spd.base,
            attack_type: atk.attack_type.clone(), attack_damage: atk.damage,
        }
    }).collect();
    let saved_items: Vec<SavedItem> = item_query.iter().map(|(_, name, data, pos)| {
        SavedItem {
            name: name.0.clone(), pos: pos.0, class: data.0.class,
            plus: data.0.plus, quantity: data.0.quantity, glyph: data.0.glyph,
        }
    }).collect();
    store.levels.insert(level.depth, SavedLevel {
        grid: terrain.clone(), monsters: saved_monsters, items: saved_items, player_pos: player.pos,
    });

    // Despawn everything
    for (entity, _, _, _, _, _, _, _, _) in monster_query.iter() { commands.entity(entity).despawn(); }
    for (entity, _, _, _) in item_query.iter() { commands.entity(entity).despawn(); }
    for entity in terrain_sprites.iter() { commands.entity(entity).despawn(); }
    *grid = MonsterGrid::default();
    *sprite_grid = TerrainSpriteGrid::default();

    // Change depth
    let new_depth = if going_down { level.depth + 1 } else { level.depth - 1 };
    level.depth = new_depth;

    if going_down {
        messages.add(format!("You descend to D:{}.", new_depth));
    } else {
        messages.add(format!("You ascend to D:{}.", new_depth));
    }

    // Load or generate level
    let new_grid = if let Some(saved) = store.levels.get(&new_depth) {
        // Restore saved level
        player.pos = saved.player_pos;
        for sm in &saved.monsters {
            let tile_id = dcss_tiles::monster_name_to_tile(&sm.name).unwrap_or(TileId::MonGoblin);
            let w = sm.pos.to_world();
            let entity = commands.spawn((
                MonsterTag, MonsterName(sm.name.clone()),
                Health { current: sm.hp, max: sm.max_hp },
                Position(sm.pos), Speed { base: sm.speed, energy: 0 },
                MeleeAttack { attack_type: sm.attack_type.clone(), damage: sm.attack_damage },
                ArmorClass(sm.ac), Evasion(sm.ev), HitDice(sm.hd),
                Sprite::from_image(tiles.get(tile_id)),
                Transform::from_xyz(w.x, w.y, 1.0),
            )).id();
            grid.set(sm.pos, Some(entity));
        }
        for si in &saved.items {
            commands.spawn((ItemTag, ItemName(si.name.clone()),
                ItemData(item::ItemDef { name: si.name.clone(), class: si.class, plus: si.plus, quantity: si.quantity, glyph: si.glyph, ..Default::default() }),
                ItemPosition(si.pos)));
        }
        saved.grid.clone()
    } else {
        // Generate new level
        let des_path = "crawl-ref/source/dat/des/arrival/simple.des";
        let mut new_grid = match load_des_vault(des_path, (new_depth as usize) % 20) {
            Ok((g, pos, name)) => { messages.add(format!("Level: {}", name)); player.pos = pos; g }
            Err(_) => { player.pos = Coord::new(5, 5); terrain::hardcoded_dungeon() }
        };
        terrain::ensure_stairs(&mut new_grid, new_depth, MAX_DEPTH);

        // Spawn monsters for new level
        spawn_monsters_for_level(&mut commands, &defs, &tiles, &new_grid, &player, &mut grid, new_depth);

        // Spawn floor items
        spawn_items_for_level(&mut commands, &new_grid, &player, &grid, new_depth, &tiles);

        // On D:5, spawn the Orb
        if new_depth == MAX_DEPTH {
            spawn_orb(&mut commands, &new_grid, &player, &grid, &tiles);
        }

        new_grid
    };

    // Spawn terrain sprites (with floor underlay for transparent features)
    for y in 0..MAP_HEIGHT { for x in 0..MAP_WIDTH {
        let pos = Coord::new(x as i32, y as i32);
        let feature = new_grid.cells[y][x];
        let wx = x as f32 * TILE_SIZE;
        let wy = -(y as f32) * TILE_SIZE;
        let needs_floor = matches!(feature,
            Feature::ClosedDoor | Feature::OpenDoor | Feature::StairsDown | Feature::StairsUp);
        if needs_floor {
            commands.spawn((Sprite::from_image(tiles.get(TileId::FloorGreyDirt)),
                Transform::from_xyz(wx, wy, 0.0)));
        }
        let z = if needs_floor { 0.1 } else { 0.0 };
        let e = commands.spawn((TerrainSpriteMarker, GridPos(pos),
            Sprite::from_image(tiles.get(feature_to_tile(feature))),
            Transform::from_xyz(wx, wy, z))).id();
        sprite_grid.set(pos, Some(e));
    }}
    commands.insert_resource(new_grid);
    player.turn_is_over = true;
    player.time_taken = 10;
}

fn spawn_monsters_for_level(commands: &mut Commands, defs: &MonsterDefs, tiles: &TileRegistry,
    terrain: &TerrainGrid, player: &Player, grid: &mut MonsterGrid, depth: i32) {
    let mut rng = rand::rng();
    let mut floor_tiles = Vec::new();
    for y in 0..MAP_HEIGHT { for x in 0..MAP_WIDTH {
        let p = Coord::new(x as i32, y as i32);
        if terrain.cells[y][x] == Feature::Floor && grid.get(p).is_none() && p != player.pos {
            floor_tiles.push(p);
        }
    }}
    let names = monsters_for_depth(depth);
    let count = (5 + depth * 2).min(floor_tiles.len() as i32) as usize;
    for i in 0..count {
        if floor_tiles.is_empty() { break }
        let idx = rng.random_range(0..floor_tiles.len());
        let pos = floor_tiles.swap_remove(idx);
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

fn spawn_items_for_level(commands: &mut Commands, terrain: &TerrainGrid, player: &Player,
    grid: &MonsterGrid, depth: i32, tiles: &TileRegistry) {
    let mut rng = rand::rng();
    let mut floor_tiles = Vec::new();
    for y in 0..MAP_HEIGHT { for x in 0..MAP_WIDTH {
        let p = Coord::new(x as i32, y as i32);
        if terrain.cells[y][x] == Feature::Floor && grid.get(p).is_none() && p != player.pos {
            floor_tiles.push(p);
        }
    }}
    let count = rng.random_range(3..=6 + depth as usize).min(floor_tiles.len());
    for _ in 0..count {
        if floor_tiles.is_empty() { break }
        let idx = rng.random_range(0..floor_tiles.len());
        let pos = floor_tiles.swap_remove(idx);
        let def = item::random_item(depth);
        let class_str = match def.class {
            item::ItemClass::Weapon => "weapon",
            item::ItemClass::Armour => "armour",
            item::ItemClass::Potion => "potion",
            item::ItemClass::Scroll => "scroll",
            item::ItemClass::Gold => "gold",
            _ => "gold",
        };
        let tile_id = dcss_tiles::item_subtype_to_tile(&def.subtype, class_str);
        let w = pos.to_world();
        commands.spawn((ItemTag, ItemName(def.name.clone()), ItemData(def), ItemPosition(pos),
            Sprite::from_image(tiles.get(tile_id)),
            Transform::from_xyz(w.x, w.y, 0.5)));
    }
}

fn spawn_orb(commands: &mut Commands, terrain: &TerrainGrid, player: &Player,
    grid: &MonsterGrid, tiles: &TileRegistry) {
    let mut rng = rand::rng();
    let mut floor_tiles = Vec::new();
    for y in 0..MAP_HEIGHT { for x in 0..MAP_WIDTH {
        let p = Coord::new(x as i32, y as i32);
        if terrain.cells[y][x] == Feature::Floor && grid.get(p).is_none() && p != player.pos {
            floor_tiles.push(p);
        }
    }}
    if let Some(&pos) = floor_tiles.first() {
        let orb = item::ItemDef {
            name: "the Orb of Zot".into(),
            class: item::ItemClass::Orb,
            glyph: '0', subtype: "orb".into(),
            ..Default::default()
        };
        let w = pos.to_world();
        commands.spawn((ItemTag, ItemName(orb.name.clone()), ItemData(orb), ItemPosition(pos),
            Sprite::from_image(tiles.get(TileId::OrbOfZot)),
            Transform::from_xyz(w.x, w.y, 0.5)));
    }
}

fn execute_player_action(mut pending: ResMut<PendingMove>, mut player: ResMut<Player>,
    mut terrain: ResMut<TerrainGrid>, mg: Res<MonsterGrid>, sg: Res<TerrainSpriteGrid>,
    tiles: Res<TileRegistry>, mut monsters: Query<(&MonsterName, &mut Health, &ArmorClass, &Evasion)>,
    mut sprites: Query<&mut Sprite, With<TerrainSpriteMarker>>, mut messages: ResMut<MessageLog>,
    mut commands: Commands, items: Query<(Entity, &ItemName, &ItemData, &ItemPosition), With<ItemTag>>,
    mut inventory: ResMut<Inventory>) {
    let Some((dx, dy)) = pending.command.take() else { return };
    let target = Coord::new(player.pos.x + dx, player.pos.y + dy);
    if !target.in_bounds() { return }
    if let Some(entity) = mg.get(target) {
        if let Ok((name, mut hp, ac, ev)) = monsters.get_mut(entity) {
            let brand = player.weapon_brand();
            let result = combat::resolve_melee(player.weapon_damage(), player.accuracy(), ac.0, ev.0, brand);
            if result.hit && result.total_damage > 0 {
                hp.current -= result.total_damage;
                let brand_msg = if result.brand_damage > 0 {
                    format!(" + {} {}", result.brand_damage, result.brand.map(|b| b.name()).unwrap_or(""))
                } else { String::new() };
                messages.add(format!("You hit the {} with your {}! ({}{} = {} damage)",
                    name.0, player.weapon_name(), result.damage, brand_msg, result.total_damage));
                // Draining heals attacker
                if result.brand == Some(item::Brand::Draining) {
                    let heal = (result.total_damage / 4).max(1);
                    player.hp = (player.hp + heal).min(player.max_hp);
                    messages.add(format!("You drain the {}! (+{} HP)", name.0, heal));
                }
                // Venom poisons (for player status - simplify to just extra message)
                if result.brand == Some(item::Brand::Venom) {
                    messages.add(format!("The {} is poisoned!", name.0));
                }
            } else if result.hit { messages.add(format!("You hit the {} but do no damage.", name.0)); }
            else { messages.add(format!("You miss the {}.", name.0)); }
            player.turn_is_over = true; player.time_taken = 10;
        }
    } else if terrain.get(target) == Some(Feature::ClosedDoor) {
        terrain.set(target, Feature::OpenDoor);
        if let Some(se) = sg.get(target) { if let Ok(mut s) = sprites.get_mut(se) { s.image = tiles.get(TileId::DoorOpen); } }
        messages.add("You open the door."); player.turn_is_over = true; player.time_taken = 10;
    } else if terrain.is_passable(target) {
        player.pos = target; player.turn_is_over = true; player.time_taken = 10;
        // Auto-pickup items at new position
        for (entity, name, data, ipos) in items.iter() {
            if ipos.0 == target {
                if data.0.class == item::ItemClass::Orb {
                    player.has_orb = true;
                    messages.add("You pick up the Orb of Zot! Now ascend to the surface!");
                } else {
                    messages.add(format!("You pick up {}.", name.0));
                    inventory.add(data.0.clone());
                    player.gold = inventory.gold;
                }
                commands.entity(entity).despawn();
            }
        }
    }
}

fn check_monster_death(mut commands: Commands,
    q: Query<(Entity, &MonsterName, &Health, &HitDice, &Position), With<MonsterTag>>,
    mut grid: ResMut<MonsterGrid>, mut messages: ResMut<MessageLog>, mut player: ResMut<Player>) {
    for (entity, name, health, hd, pos) in &q {
        if health.current <= 0 {
            let xp_gain = hd.0 * 10;
            messages.add(format!("The {} dies! (+{} XP)", name.0, xp_gain));
            grid.set(pos.0, None);
            commands.entity(entity).despawn();
            player.xp += xp_gain;
            player.kills += 1;
            // Level up check
            while player.xp >= player.xp_next {
                player.xl += 1;
                player.xp -= player.xp_next;
                player.xp_next = player.xl * 25;
                player.max_hp += 3 + player.str_stat / 4;
                player.hp = player.max_hp;
                player.max_mp += 1;
                player.mp = player.max_mp;
                messages.add(format!("Welcome to experience level {}!", player.xl));
            }
        }
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
            if let Some(next_step) = pathfind::astar_next_step(pos.0, pp, &terrain, &grid) {
                if next_step != pp && terrain.is_passable(next_step) && grid.get(next_step).is_none() {
                    moves.push((entity, pos.0, next_step));
                }
            }
        }
    }

    // Apply attacks
    for (name, dmg, _atype, hd) in &attacks {
        let result = combat::resolve_melee(*dmg, hd * 3 + 5, player.total_ac(), player.ev, None);
        if result.hit && result.total_damage > 0 { player.hp -= result.total_damage; messages.add(format!("The {} hits you! ({} damage)", name, result.total_damage)); }
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

    // Tick status effects and regen
    let status_msgs = player.tick_status();
    for msg in status_msgs {
        messages.add(msg);
    }

    player.turn_is_over = false;
}

fn render_death_screen_system(mut contexts: EguiContexts, mut state: ResMut<LituiState>,
    player: Res<Player>, level: Res<CurrentLevel>, game_mode: Res<State<GameMode>>) -> Result {
    let is_win = **game_mode == GameMode::Won;
    state.death_title = if is_win { "Victory!".into() } else { "Game Over".into() };
    state.death_cause = if is_win {
        "You escaped the dungeon with the Orb of Zot!".into()
    } else {
        "You have been slain.".into()
    };
    state.death_species = player.species_name.clone();
    state.death_job = player.job_name.clone();
    state.death_xl = format!("{}", player.xl);
    state.death_turns = format!("{}", player.turns);
    state.death_depth = format!("D:{}", level.depth);
    state.death_gold = format!("{}", player.gold);
    state.death_kills = format!("{}", player.kills);
    state.death_weapon = player.weapon_name();
    state.death_armour = player.armour_name();

    egui::CentralPanel::default().show(contexts.ctx_mut()?, |ui| {
        render_death_screen(ui, &mut state);
    });
    Ok(())
}

fn death_screen_input(keyboard: Res<ButtonInput<KeyCode>>, mut app_exit: MessageWriter<AppExit>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        app_exit.write(AppExit::Success);
    }
    // TODO: Enter to restart (requires full state reset)
}

fn check_player_death(player: Res<Player>, mut messages: ResMut<MessageLog>, mut ns: ResMut<NextState<GameMode>>) {
    if player.hp <= 0 { messages.add("You die..."); messages.add("Press Escape to quit."); ns.set(GameMode::GameOver); }
}

fn load_monster_defs(mut defs: ResMut<MonsterDefs>) {
    let base = "crawl-ref/source/dat/mons";
    let names = [
        "goblin", "kobold", "rat", "bat", "jackal", "gnoll", "orc",
        "adder", "hobgoblin", "ogre", "scorpion", "troll", "yak",
    ];
    for name in names {
        if let Ok(c) = std::fs::read_to_string(format!("{}/{}.yaml", base, name)) {
            if let Ok(def) = serde_yaml::from_str::<MonsterDef>(&c) { defs.0.insert(name.into(), def); }
        }
    }
}

/// Monsters appropriate for each depth.
fn monsters_for_depth(depth: i32) -> &'static [&'static str] {
    match depth {
        1 => &["rat", "bat", "goblin", "kobold"],
        2 => &["jackal", "gnoll", "adder", "hobgoblin", "kobold"],
        3 => &["orc", "ogre", "scorpion", "gnoll"],
        4 => &["orc", "troll", "yak", "ogre"],
        _ => &["troll", "ogre", "yak", "orc"],
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
