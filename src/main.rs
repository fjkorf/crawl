mod plugin;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use plugin::DcssGamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "DCSS Bevy".into(),
                resolution: bevy::window::WindowResolution::new(1280, 960),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(DcssGamePlugin)
        .run();
}
