mod plugin;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use plugin::{DcssGamePlugin, DungeonSource};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "DCSS Bevy".into(),
            resolution: bevy::window::WindowResolution::new(1280, 960),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(EguiPlugin::default())
    .add_plugins(DcssGamePlugin);

    // --des <file> [index] loads a vault from a .des file
    if let Some(pos) = args.iter().position(|a| a == "--des") {
        if let Some(file) = args.get(pos + 1) {
            let index = args
                .get(pos + 2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            app.insert_resource(DungeonSource::DesVault {
                des_file: file.clone(),
                vault_index: index,
            });
        }
    }

    app.run();
}
