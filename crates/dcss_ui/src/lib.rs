//! DCSS UI — all game screens rendered via litui markdown.

pub mod examine;

// --- litui multi-page app ---

pub mod pages {
    use egui;
    use litui::*;

    define_markdown_app! {
        parent: "content/_app.md",
        "content/chargen.md",
        "content/inventory.md",
        "content/monster_info.md",
        "content/stat_panel.md",
        "content/message_log.md",
        "content/death_screen.md",
    }
}

pub use pages::{AppState as LituiState, Page as LituiPage};
pub use pages::{render_chargen, render_inventory, render_monster_info, render_stat_panel, render_message_log, render_death_screen};

impl bevy_ecs::prelude::Resource for LituiState {}
impl bevy_ecs::prelude::Resource for LituiPage {}
