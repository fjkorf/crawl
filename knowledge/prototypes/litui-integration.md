# litui Integration Findings

## What Worked

- litui's `define_markdown_app!` macro compiles markdown to egui widgets at build time with zero runtime parsing
- The Bevy integration pattern (`EguiPlugin` + `EguiPrimaryContextPass` + Resources) works cleanly
- Styles via YAML frontmatter are powerful for theming
- The demo_content example was an excellent reference for project structure

## Option C Implemented — Display-Only Fields Work

litui implemented Option C from our proposal: `[display](field)` now self-declares its field as `String` on `AppState` when no other widget provides it. The monster info popup is now rendered entirely via litui markdown:

```markdown
# [display](monster_name) {title}

| Stat | Value |
|------|-------|
| **HD** | [display](hd) |
| **HP** | [display](hp) |
```

The system populates `info_state.monster_name`, `.hd`, `.hp`, etc. from ECS components each frame, and litui renders them as a styled markdown table inside an `egui::Window`.

## Integration Pattern

```rust
mod monster_page {
    use litui::*;
    define_markdown_app! {
        parent: "content/_app.md",
        "content/monster_info.md",
    }
}
pub use monster_page::AppState as MonsterInfoState;
impl Resource for MonsterInfoState {}
```

Then in the system: populate `info_state` fields and call `monster_page::render_monster_info(ui, &mut info_state)`.

## When to Use litui in This Project

- **Good fit**: Monster info popups, character dump, help screens, god descriptions, species/background selection
- **Pattern**: Any screen where content is text-heavy and data is injected via `[display]` fields
- **Not a fit**: The dungeon grid, minimap, real-time animated elements
