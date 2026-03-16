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

- **Good fit**: Monster info popups, character dump, help screens, god descriptions
- **Pattern**: Any screen where content is text-heavy and data is injected via `[display]` fields
- **Not a fit**: The dungeon grid, minimap, real-time animated elements

## Additional Limitations Found

### Character Creation Screen — Dynamic Lists
The species/job selection screen needs to display a list of items loaded from YAML at runtime. litui's `define_markdown_app!` generates a fixed `Page` enum at compile time — there's no way to iterate over a dynamic collection or render a selectable list from runtime data.

**What litui would need**: A `[list]` or `[select]` widget directive that takes an array field from AppState and renders it as a selectable list:
```markdown
[select](selected_species){species_list}
```
Where `species_list` is a `Vec<String>` on AppState and `selected_species` is a `usize` index.

**Workaround**: Plain egui with `selectable_label` in a loop.

### Inventory Screen — Dynamic Item Lists
Same pattern as character creation: the inventory is a `Vec<ItemDef>` that changes at runtime. litui can't iterate over dynamic collections.

**Workaround**: Plain egui with `ScrollArea` and a for loop.

### Summary of litui Gaps

| Use Case | Blocker | Workaround |
|----------|---------|-----------|
| Read-only data popup | `[display]` needed self-declaration | **Fixed** (Option C implemented) |
| Dynamic list selection | No `[list]`/`[select]` widget | Plain egui |
| Runtime-length collections | Markdown is compile-time fixed | Plain egui |

litui excels at **static content with injected values**. It doesn't yet support **dynamic-length data** or **iterative rendering**.
