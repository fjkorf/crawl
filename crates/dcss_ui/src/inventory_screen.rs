//! Inventory screen using egui.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use dcss_core::item::Inventory;

pub fn inventory_screen_system(
    mut contexts: EguiContexts,
    inventory: Res<Inventory>,
) -> Result {
    egui::Window::new("Inventory")
        .collapsible(false)
        .resizable(true)
        .default_width(350.0)
        .default_height(400.0)
        .show(contexts.ctx_mut()?, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Inventory");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("Gold: {}", inventory.gold))
                        .color(egui::Color32::YELLOW));
                });
            });
            ui.separator();

            if inventory.items.is_empty() {
                ui.label("You are empty-handed.");
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, item) in inventory.items.iter().enumerate() {
                        let letter = (b'a' + i as u8) as char;
                        let label = if item.quantity > 1 {
                            format!("{}) {} (x{})", letter, item.name, item.quantity)
                        } else {
                            format!("{}) {}", letter, item.name)
                        };
                        ui.label(&label);
                    }
                });
            }

            ui.separator();
            ui.label("Press i or Escape to close.");
        });
    Ok(())
}
