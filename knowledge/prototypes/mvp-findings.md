# MVP Prototype Findings

## Critical: Bevy 0.18 Feature Flags

`bevy_sprite` alone is NOT enough for sprite rendering. You MUST also enable `bevy_sprite_render`. In Bevy 0.18, API crates and render crates are separate:
- `bevy_sprite` = types, components, API
- `bevy_sprite_render` = actual GPU rendering pipeline

Minimal feature set for a 2D sprite game with egui:
```toml
bevy_winit, bevy_render, bevy_core_pipeline,
bevy_sprite, bevy_sprite_render,
bevy_text, bevy_asset, bevy_state, bevy_image,
png, default_font, x11
```

## Screenshot Testing

Bevy 0.18 provides `Screenshot::primary_window()` + `save_to_disk()` observer pattern. Key findings:
- `MessageWriter<AppExit>` replaces `EventWriter<AppExit>` for app exit
- Need ~15 frames for assets to load and scene to stabilize before capture
- Screenshot saves asynchronously — wait ~10 more frames before exiting
- Comparison test runs the example as a subprocess, loads both PNGs with `image` crate, does per-pixel RGBA diff with channel threshold of 30 and max 2% pixel difference allowed

## DcssPlugin Pattern

Extracted all shared game setup into `crates/dcss_game/src/lib.rs::DcssGamePlugin` (originally `src/plugin.rs`, moved to its own crate). Both `main.rs` (~20 lines) and `examples/screenshot_test.rs` (~170 lines) use it. This avoids ~200 lines of duplication and makes test examples trivial to write.

Key design: the plugin owns all game systems but does NOT own the window/rendering plugins. The binary or example adds `DefaultPlugins` + `EguiPlugin` + `DcssGamePlugin`.

## Walkthrough Screenshot Test

Scripted movement through all 4 dungeon rooms, taking a screenshot at each stop. Uses:
- `WalkthroughQueue` resource with `VecDeque<WalkthroughStep>` (Move/Wait/Screenshot)
- System runs in `First` schedule to set `PendingMove` before the plugin's input system
- `player_input` must NOT overwrite `PendingMove` when no key is pressed — changed to only write on actual input

Produces 4 baseline PNGs (room1-room4) covering all 7 monsters + doors + stairs.

## Door Interaction

`TerrainGrid::set()` mutates the grid, `TerrainSpriteGrid` maps coords to sprite entities for swapping tile images. Opening a closed door updates both the data grid and the visual sprite in one system.

## Bevy 0.18 API Notes

### Breaking Changes from Common Examples
- `WindowResolution::new()` takes `(u32, u32)` not `(f32, f32)`
- `add_event::<T>()` removed — events no longer need explicit registration. However, `EventWriter`/`EventReader` may also be removed or renamed. **Workaround**: Use a `Resource` with `Option<T>` instead of events for simple command passing.
- `bevy_ecs`, `bevy_state`, etc. are NOT features of the `bevy` crate — they're always included. Using `default-features = false` with no feature flags works for library crates.
- Sub-crate dependencies: library crates can depend on `bevy` with `default-features = false` and get ECS, asset system, etc. Only the root binary needs rendering features.

### Feature Flags for Root Binary
Minimal set for a 2D sprite game with egui:
```toml
bevy_winit, bevy_render, bevy_core_pipeline, bevy_sprite, bevy_text,
bevy_asset, bevy_state, png, default_font, x11
```

### System Scheduling
- `.chain()` works on tuples but cascading type errors make debugging hard
- Better to split into separate `.add_systems()` calls with explicit `.after()` ordering
- `run_if(in_state(GameMode::Play))` works as expected

### Resource-Based Command Passing
Instead of events (which had API issues in 0.18), use:
```rust
#[derive(Resource, Default)]
pub struct PendingMove {
    pub command: Option<(i32, i32)>,
}
```
Input system sets it, action system takes it with `.take()`. Simple and reliable.

## Tile Loading
- Individual PNGs from `rltiles/` load via `AssetServer` with no issues
- Symlink `assets/rltiles → crawl-ref/source/rltiles` works for Bevy's asset path resolution
- 32x32 tiles render correctly with `Sprite::from_image()`
- Z-ordering: terrain at 0.0, player/monsters at 1.0

## Monster YAML
- `serde_yaml` deserializes DCSS monster YAML directly with minimal struct definitions
- Optional fields handled with `#[serde(default)]`
- The YAML schema is simple and consistent across all monster files tested (goblin, rat, orc, gnoll, kobold, bat, jackal)

## ECS Component Design
- Separate fine-grained components (Health, Position, Speed, etc.) work well with Bevy queries
- `MonsterGrid` resource for O(1) spatial lookup is essential — without it, checking "is there a monster at this coord" would require iterating all monsters
- Grid sync is manual (must update on spawn, death, move) — potential bug source

## egui Integration
- `EguiPlugin::default()` + systems in `EguiPrimaryContextPass` schedule works as documented
- `SidePanel::right` and `TopBottomPanel::bottom` coexist correctly
- egui panels automatically reserve screen space, affecting the central viewport
