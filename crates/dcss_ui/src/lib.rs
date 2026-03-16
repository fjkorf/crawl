//! DCSS UI screens — litui markdown pages + egui panels.

pub mod stat_panel;
pub mod message_panel;
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
    }
}

pub use pages::{AppState as LituiState, Page as LituiPage};
pub use pages::{render_chargen, render_inventory, render_monster_info};

impl bevy_ecs::prelude::Resource for LituiState {}
impl bevy_ecs::prelude::Resource for LituiPage {}
