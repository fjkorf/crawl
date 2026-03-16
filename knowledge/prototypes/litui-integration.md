# litui Integration — Final Status

## 100% litui for All Game UI Content (5 Pages)

| Screen | litui Features | Markdown |
|--------|---------------|----------|
| Character Creation | `[select]` × 2, `[display]`, `[button]` | chargen.md |
| Inventory | `::: foreach`, `[display]` | inventory.md |
| Monster Info | `[display]` × 7 | monster_info.md |
| Stat Panel | `[progress]`, `::: style`, `::: if`, `[display]` | stat_panel.md |
| Message Log | `[log]` | message_log.md |

**Zero egui widget code** — egui only provides structural containers (SidePanel, Window, etc.). All game content authored in ~116 lines of markdown.

Uses `::key` syntax (not old `{key}`) for style application. Widget config `{config}` unchanged.

## Integration Pattern

All three pages are in a single `define_markdown_app!`:
```rust
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
```

The generated `AppState` has fields from ALL pages merged into one struct. Each render system populates the relevant fields before calling `render_*()`.

## Key Patterns

### Populating [select] lists from Bevy Resources
```rust
fn populate_chargen_state(species: Res<SpeciesDefs>, mut state: ResMut<LituiState>) {
    if state.species_list.is_empty() {
        state.species_list = species.0.iter().map(|s| s.name.clone()).collect();
    }
    // Read state.chosen_species as usize index
}
```

### Populating [foreach] rows from game state
```rust
fn populate_inventory(inventory: Res<Inventory>, mut state: ResMut<LituiState>) {
    state.items.clear();
    for (i, item) in inventory.items.iter().enumerate() {
        let mut row = pages::ItemsRow::default();
        row.letter = format!("{}", (b'a' + i as u8) as char);
        row.name = item.name.clone();
        state.items.push(row);
    }
}
```

### Button click detection
```rust
if litui_state.start_game_count > 0 {
    litui_state.start_game_count = 0;
    // Handle button click
}
```

## Issues Resolved

All 7 issues from the integration issues doc were addressed by litui:
1. `[foreach]` in `define_markdown_app!` — fixed (WidgetField::Foreach refactoring)
2. `[select]` in `define_markdown_app!` — confirmed working
3. Row type naming — documented: `capitalize_first(name) + "Row"`, public, Default
4. Widget-in-table support — documented (select NOT supported in cells)
5. Button field naming — documented: `{config}_count`, `{config}_hovered`
6. Error messages — improved with file/line context
7. Game demo example — added to litui

## Syntax Requirements Discovered

- `[foreach]` body MUST have a blank line before the table (pulldown-cmark paragraph separation)
- `[select]` works standalone but NOT inside table cells
- Button labels use underscores for spaces: `[button](Start_Game)` not `[button](Start Game)`
- Stale proc-macro build caches can cause phantom errors — `cargo clean` resolves

## Screens Remaining as egui

- **Stat panel** — uses `ProgressBar` widget which litui doesn't have
- **Message log** — real-time scrolling append, not a litui pattern
- **Dungeon grid / minimap / cursor** — Bevy 2D sprites, not UI
