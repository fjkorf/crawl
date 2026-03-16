use bevy::prelude::*;
use bevy_ecs::prelude::Resource;
use bevy_egui::{EguiContexts, egui};

use dcss_core::monster::*;
use dcss_core::types::Coord;

// --- litui-generated monster info page ---

mod monster_page {
    use egui;
    use litui::*;

    define_markdown_app! {
        parent: "content/_app.md",
        "content/monster_info.md",
    }
}

pub use monster_page::AppState as MonsterInfoState;

impl Resource for MonsterInfoState {}
impl Resource for monster_page::Page {}

// --- Examine cursor ---

#[derive(Resource)]
pub struct ExamineCursor {
    pub pos: Coord,
    pub showing_popup: bool,
}

impl Default for ExamineCursor {
    fn default() -> Self {
        Self {
            pos: Coord::new(0, 0),
            showing_popup: false,
        }
    }
}

/// Marker for the cursor highlight sprite.
#[derive(Component)]
pub struct ExamineCursorSprite;

// --- Systems ---

/// Handle input in Examine mode: move cursor, Enter to show/hide popup, Escape to exit.
pub fn examine_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cursor: ResMut<ExamineCursor>,
    monster_grid: Res<MonsterGrid>,
    mut messages: ResMut<dcss_core::message::MessageLog>,
    monsters: Query<&MonsterName, With<MonsterTag>>,
    mut next_state: ResMut<NextState<dcss_core::turn::GameMode>>,
) {
    let delta = if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyH)
    {
        Some((-1, 0))
    } else if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyL) {
        Some((1, 0))
    } else if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyK) {
        Some((0, -1))
    } else if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyJ) {
        Some((0, 1))
    } else {
        None
    };

    if let Some((dx, dy)) = delta {
        let new_pos = Coord::new(cursor.pos.x + dx, cursor.pos.y + dy);
        if new_pos.in_bounds() {
            cursor.pos = new_pos;
            cursor.showing_popup = false;

            if let Some(entity) = monster_grid.get(new_pos) {
                if let Ok(name) = monsters.get(entity) {
                    messages.add(format!("You see a {}.", name.0));
                }
            }
        }
    }

    if keyboard.just_pressed(KeyCode::Enter) {
        if monster_grid.get(cursor.pos).is_some() {
            cursor.showing_popup = !cursor.showing_popup;
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        cursor.showing_popup = false;
        next_state.set(dcss_core::turn::GameMode::Play);
    }
}

/// Render the monster info popup using litui-generated markdown.
pub fn examine_popup_system(
    mut contexts: EguiContexts,
    cursor: Res<ExamineCursor>,
    monster_grid: Res<MonsterGrid>,
    monsters: Query<(
        &MonsterName,
        &Health,
        &HitDice,
        &ArmorClass,
        &Evasion,
        &Speed,
        &MeleeAttack,
    )>,
    mut info_state: ResMut<MonsterInfoState>,
) -> Result {
    if !cursor.showing_popup {
        return Ok(());
    }

    let Some(entity) = monster_grid.get(cursor.pos) else {
        return Ok(());
    };

    let Ok((name, hp, hd, ac, ev, speed, attack)) = monsters.get(entity) else {
        return Ok(());
    };

    // Populate litui AppState with monster data
    info_state.monster_name = name.0.clone();
    info_state.hd = format!("{}", hd.0);
    info_state.hp = format!("{}/{}", hp.current, hp.max);
    info_state.ac = format!("{}", ac.0);
    info_state.ev = format!("{}", ev.0);
    info_state.speed = format!("{}", speed.base);
    info_state.attack = format!("{} ({})", attack.attack_type, attack.damage);

    egui::Window::new("Monster Info")
        .collapsible(false)
        .resizable(false)
        .default_width(280.0)
        .show(contexts.ctx_mut()?, |ui| {
            monster_page::render_monster_info(ui, &mut info_state);
        });

    Ok(())
}

/// Sync the cursor highlight sprite position.
pub fn examine_cursor_sync(
    cursor: Res<ExamineCursor>,
    mut query: Query<(&mut Transform, &mut Visibility), With<ExamineCursorSprite>>,
) {
    for (mut transform, mut visibility) in &mut query {
        let world = cursor.pos.to_world();
        transform.translation.x = world.x;
        transform.translation.y = world.y;
        *visibility = Visibility::Visible;
    }
}

/// Hide the cursor sprite when not in examine mode.
pub fn hide_examine_cursor(mut query: Query<&mut Visibility, With<ExamineCursorSprite>>) {
    for mut visibility in &mut query {
        *visibility = Visibility::Hidden;
    }
}
