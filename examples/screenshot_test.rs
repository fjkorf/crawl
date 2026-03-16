//! Screenshot test: renders the dungeon scene, optionally walks through all rooms,
//! captures screenshots, and exits.
//!
//! Modes (set via SCREENSHOT_MODE env var):
//!   static      — single screenshot at spawn position (default)
//!   walkthrough — scripted walk visiting all 4 rooms, 4 screenshots
//!
//! Usage:
//!   cargo run --example screenshot_test
//!   SCREENSHOT_MODE=walkthrough cargo run --example screenshot_test

use std::collections::VecDeque;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy_egui::EguiPlugin;

use dcss_core::turn::{GameMode, PendingMove};
use dcss_core::types::Coord;
use dcss_game::{DcssGamePlugin, DungeonSource};
use dcss_ui::examine::ExamineCursor;

// --- Separate, focused resources ---

#[derive(Resource, Default)]
struct FrameCount(u32);

#[derive(Resource)]
struct WalkthroughQueue(VecDeque<WalkthroughStep>);

#[derive(Clone)]
enum WalkthroughStep {
    Move(i32, i32),
    WaitFrames(u32),
    Screenshot(String),
    EnterExamine,
    ExamineAt(i32, i32),
    ExamineOpen,
    ExitExamine,
}

#[derive(Resource)]
struct StaticScreenshotPath(String);

/// Counts down wait frames. Inserted/removed dynamically.
#[derive(Resource)]
struct WaitTimer(u32);

fn main() {
    let mode = std::env::var("SCREENSHOT_MODE").unwrap_or_else(|_| "static".into());
    let output_dir = std::env::var("SCREENSHOT_DIR").unwrap_or_else(|_| "tests/snapshots".into());

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "DCSS Screenshot Test".into(),
            resolution: bevy::window::WindowResolution::new(1280, 960),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(EguiPlugin::default())
    .add_plugins(DcssGamePlugin);

    // DES_FILE env var loads a vault from a .des file
    if let Ok(des_file) = std::env::var("DES_FILE") {
        let vault_index: usize = std::env::var("DES_VAULT_INDEX")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        app.insert_resource(DungeonSource::DesVault {
            des_file,
            vault_index,
        });
    }

    // Skip character creation for automated tests
    {
        use dcss_core::chargen::ChargenState;
        use dcss_core::turn::GamePhase;
        app.insert_resource(ChargenState { species_index: 0, job_index: 0, confirmed: true });
    }

    app.init_resource::<FrameCount>()
    .add_systems(Update, tick_frame);

    match mode.as_str() {
        "walkthrough" => {
            app.insert_resource(WalkthroughQueue(build_walkthrough(&output_dir)));
            // Must run before the plugin's player_input system which also writes PendingMove
            app.add_systems(First, walkthrough_system);
        }
        _ => {
            let path = std::env::var("SCREENSHOT_OUTPUT")
                .unwrap_or_else(|_| format!("{}/dungeon_scene.new.png", output_dir));
            app.insert_resource(StaticScreenshotPath(path));
            app.add_systems(Update, static_screenshot_system.after(tick_frame));
        }
    }

    app.run();
}

fn tick_frame(mut count: ResMut<FrameCount>) {
    count.0 += 1;
}

// --- Static mode: wait for scene, take one screenshot, exit ---

fn static_screenshot_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    path: Res<StaticScreenshotPath>,
    mut exit: MessageWriter<AppExit>,
    mut taken: Local<bool>,
) {
    if frame.0 == 15 && !*taken {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.0.clone()));
        *taken = true;
    }
    if frame.0 >= 25 {
        exit.write(AppExit::Success);
    }
}

// --- Walkthrough mode: process step queue one step per frame ---

fn walkthrough_system(
    mut commands: Commands,
    mut queue: ResMut<WalkthroughQueue>,
    mut pending: ResMut<PendingMove>,
    mut cursor: ResMut<ExamineCursor>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut exit: MessageWriter<AppExit>,
    mut wait: Local<u32>,
) {
    // If waiting, count down
    if *wait > 0 {
        *wait -= 1;
        return;
    }

    let Some(step) = queue.0.pop_front() else {
        // Queue exhausted — wait a few frames for last screenshot to save, then exit
        *wait = 10;
        // Insert a sentinel so we don't keep re-entering this branch
        queue.0.push_back(WalkthroughStep::WaitFrames(0));
        exit.write(AppExit::Success);
        return;
    };

    match step {
        WalkthroughStep::Move(dx, dy) => {
            pending.command = Some((dx, dy));
        }
        WalkthroughStep::WaitFrames(n) => {
            *wait = n;
        }
        WalkthroughStep::Screenshot(path) => {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path));
        }
        WalkthroughStep::EnterExamine => {
            next_state.set(GameMode::Examine);
            cursor.showing_popup = false;
        }
        WalkthroughStep::ExamineAt(x, y) => {
            cursor.pos = Coord::new(x, y);
            cursor.showing_popup = false;
        }
        WalkthroughStep::ExamineOpen => {
            cursor.showing_popup = true;
        }
        WalkthroughStep::ExitExamine => {
            cursor.showing_popup = false;
            next_state.set(GameMode::Play);
        }
    }
}

// --- Walkthrough path definition ---

fn build_walkthrough(output_dir: &str) -> VecDeque<WalkthroughStep> {
    use WalkthroughStep::*;
    let mut s = VecDeque::new();

    // Wait for scene to load and render
    s.push_back(WaitFrames(20));

    // Room 1 screenshot — player starts at (5,5), goblin at (8,4)
    s.push_back(Screenshot(format!("{}/room1.png", output_dir)));
    s.push_back(WaitFrames(5));

    // Examine the goblin at (8,4)
    s.push_back(EnterExamine);
    s.push_back(WaitFrames(2));
    s.push_back(ExamineAt(8, 4));
    s.push_back(ExamineOpen);
    s.push_back(WaitFrames(3));
    s.push_back(Screenshot(format!("{}/examine_goblin.png", output_dir)));
    s.push_back(WaitFrames(3));
    s.push_back(ExitExamine);
    s.push_back(WaitFrames(2));

    // Walk right from (5,5) toward door at (13,5): 7 steps to (12,5)
    for _ in 0..7 {
        s.push_back(Move(1, 0));
    }
    // Open door at (13,5)
    s.push_back(Move(1, 0));
    // Walk through corridor into Room 2: (14,5) to (25,5)
    for _ in 0..11 {
        s.push_back(Move(1, 0));
    }
    s.push_back(WaitFrames(3));

    // Room 2 screenshot — player at ~(25,5), rat at (25,4), kobold at (30,6)
    s.push_back(Screenshot(format!("{}/room2.png", output_dir)));
    s.push_back(WaitFrames(5));

    // Walk to corridor entrance: right to x=28, then down
    for _ in 0..3 {
        s.push_back(Move(1, 0));
    }
    // Down corridor y=5 to y=14: 9 steps
    for _ in 0..9 {
        s.push_back(Move(0, 1));
    }
    s.push_back(WaitFrames(3));

    // Room 4 screenshot — player at ~(28,14), jackal at (25,14), bat at (33,15)
    s.push_back(Screenshot(format!("{}/room4.png", output_dir)));
    s.push_back(WaitFrames(5));

    // Walk to Room 3 via corridor: down 1 to y=15, left to (8,15)
    s.push_back(Move(0, 1));
    for _ in 0..20 {
        s.push_back(Move(-1, 0));
    }
    s.push_back(WaitFrames(3));

    // Room 3 screenshot — player at ~(8,15), gnoll at (5,15), orc at (10,14)
    s.push_back(Screenshot(format!("{}/room3.png", output_dir)));
    s.push_back(WaitFrames(10));

    s
}
