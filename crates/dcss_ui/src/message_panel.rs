use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use dcss_core::message::MessageLog;

pub fn message_panel_system(
    mut contexts: EguiContexts,
    messages: Res<MessageLog>,
) -> Result {
    egui::TopBottomPanel::bottom("messages")
        .resizable(true)
        .default_height(120.0)
        .show(contexts.ctx_mut()?, |ui| {
            ui.heading("Messages");
            ui.separator();
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for msg in &messages.messages {
                        ui.label(msg);
                    }
                });
        });
    Ok(())
}
