# litui Integration Findings

## What Worked

- litui's `define_markdown_app!` macro compiles markdown to egui widgets at build time with zero runtime parsing
- The Bevy integration pattern (`EguiPlugin` + `EguiPrimaryContextPass` + Resources) works cleanly
- Styles via YAML frontmatter are powerful for theming
- The demo_content example was an excellent reference for project structure

## What We Needed But Couldn't Use litui For

### Read-Only Display Popups

Our examine mode shows a monster stat card: name, HD, HP, AC, EV, speed, attack. All fields are read-only strings populated from ECS component data each frame.

litui's `[display](field)` widget reads from AppState, which is perfect — BUT it requires the field to be *declared* by an input widget (`[slider]`, `[textedit]`, `[checkbox]`, etc.) on some page. There is no way to declare a display-only String field.

**Error we hit:**
```
[display](monster_name) references unknown field 'monster_name' —
no widget declares this field.
```

**Workaround used:** Plain egui with `egui::Window` + `egui::Grid` + `stat_row()` helper. Works fine but loses litui's markdown authoring benefits.

## Feature Request for litui

See `knowledge/prototypes/litui-display-only-proposal.md` for a detailed proposal.

## When to Use litui in This Project

- **Good fit**: Character dump, help screens, god descriptions, species/background selection — text-heavy semi-static pages where authors want to write content in markdown
- **Not a fit (currently)**: Data-driven popups where all fields come from game state and no user input is involved
- **Would become a fit**: If litui adds display-only field declarations (see proposal)
