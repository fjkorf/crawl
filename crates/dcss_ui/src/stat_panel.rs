use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use dcss_core::player::Player;

pub fn stat_panel_system(
    mut contexts: EguiContexts,
    player: Res<Player>,
) -> Result {
    egui::SidePanel::right("stats")
        .resizable(false)
        .exact_width(180.0)
        .show(contexts.ctx_mut()?, |ui| {
            ui.heading("Stats");
            ui.separator();

            // HP
            let hp_frac = player.hp as f32 / player.max_hp.max(1) as f32;
            ui.horizontal(|ui| {
                ui.label("HP:");
                let color = if hp_frac > 0.5 {
                    egui::Color32::from_rgb(0, 200, 0)
                } else if hp_frac > 0.25 {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::RED
                };
                ui.colored_label(color, format!("{}/{}", player.hp, player.max_hp));
            });
            ui.add(
                egui::ProgressBar::new(hp_frac)
                    .fill(egui::Color32::DARK_RED),
            );

            // MP
            let mp_frac = player.mp as f32 / player.max_mp.max(1) as f32;
            ui.horizontal(|ui| {
                ui.label("MP:");
                ui.colored_label(
                    egui::Color32::from_rgb(100, 100, 255),
                    format!("{}/{}", player.mp, player.max_mp),
                );
            });
            ui.add(
                egui::ProgressBar::new(mp_frac)
                    .fill(egui::Color32::from_rgb(30, 30, 100)),
            );

            ui.separator();
            ui.label(format!("AC: {}", player.ac));
            ui.label(format!("EV: {}", player.ev));
            ui.separator();
            ui.label(format!("Str: {}", player.str_stat));
            ui.label(format!("Int: {}", player.int_stat));
            ui.label(format!("Dex: {}", player.dex_stat));
            ui.separator();
            ui.label(format!("XL: {}", player.xl));
            ui.label(format!("Gold: {}", player.gold));
        });
    Ok(())
}
