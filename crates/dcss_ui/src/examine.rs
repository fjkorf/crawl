use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use dcss_core::monster::*;
use dcss_core::types::Coord;

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

/// Render the monster info popup using egui directly.
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

    egui::Window::new("Monster Info")
        .collapsible(false)
        .resizable(false)
        .default_width(260.0)
        .show(contexts.ctx_mut()?, |ui| {
            ui.colored_label(
                egui::Color32::from_rgb(255, 170, 0),
                egui::RichText::new(&name.0).size(22.0).strong(),
            );
            ui.separator();

            egui::Grid::new("monster_stats")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    stat_row(ui, "HD", &format!("{}", hd.0));
                    stat_row(
                        ui,
                        "HP",
                        &format!("{}/{}", hp.current, hp.max),
                    );
                    stat_row(ui, "AC", &format!("{}", ac.0));
                    stat_row(ui, "EV", &format!("{}", ev.0));
                    stat_row(ui, "Speed", &format!("{}", speed.base));
                    stat_row(
                        ui,
                        "Attack",
                        &format!("{} ({})", attack.attack_type, attack.damage),
                    );
                });
        });

    Ok(())
}

fn stat_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.label(egui::RichText::new(label).strong().color(egui::Color32::GRAY));
    ui.label(value);
    ui.end_row();
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
