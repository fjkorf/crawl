# litui: Comprehensive Feature Request for Full Game UI Coverage

## Context

We've migrated 3 of 6 UI screens to litui (chargen, inventory, monster info). The remaining 3 screens — stat panel, message log, and wrapper/container code — stay as egui because litui lacks specific capabilities. This proposal covers everything needed to reach 100% litui coverage for game UI content, reducing egui to only structural containers (panels, windows).

We also look ahead to screens the vertical slice needs (win/death, item selection) and future DCSS screens (god, spell, skill).

## Current State

| Screen | Approach | Why |
|--------|----------|-----|
| Character Creation | **litui** [select] + [display] + [button] | Working |
| Inventory | **litui** [foreach] + [display] | Working |
| Monster Info | **litui** [display] | Working |
| Stat Panel | **egui** ProgressBar, colored_label, conditional logic | Missing features |
| Message Log | **egui** ScrollArea with append-only list | Missing features |
| Container wrappers | **egui** SidePanel, TopBottomPanel, Window | Not litui's job (or is it?) |

## Remaining egui Code — What It Does and Why

### 1. Stat Panel (stat_panel.rs — 65 lines)

**egui widgets used**:
- `ProgressBar::new(frac).fill(color)` — HP/MP bars with custom fill colors
- `colored_label(color, text)` — HP text changes color based on percentage
- Conditional rendering — "ORB OF ZOT" only shown when `player.has_orb`

**litui blockers**:
- `[progress]` is stateless (literal f32, not AppState field)
- No conditional styling (color depends on runtime value)
- No conditional visibility (section shown/hidden based on bool)

### 2. Message Log (message_panel.rs — 24 lines)

**egui widgets used**:
- `ScrollArea::vertical().stick_to_bottom(true)` — always shows newest messages
- For loop over `messages.messages` Vec<String> — dynamic length list

**litui blockers**:
- `[foreach]` could iterate the messages, BUT it generates a table/list — not a plain scrolling text area
- No `stick_to_bottom` scroll behavior — litui has no scroll control
- Messages are plain strings, not structured rows with fields — `[foreach]` requires `{field}` references

### 3. Container Wrappers (in dcss_game/src/lib.rs and examine.rs)

Every litui render call is wrapped in an egui container:
```rust
egui::CentralPanel::default().show(ctx, |ui| { render_chargen(ui, &mut state); });
egui::Window::new("Inventory").show(ctx, |ui| { render_inventory(ui, &mut state); });
egui::Window::new("Monster Info").show(ctx, |ui| { render_monster_info(ui, &mut state); });
egui::SidePanel::right("stats").show(ctx, |ui| { ... });
egui::TopBottomPanel::bottom("messages").show(ctx, |ui| { ... });
```

**litui blocker**: litui renders content INTO a `&mut egui::Ui` but doesn't create the container. The panel/window creation must be done in Rust. This is arguably correct separation (layout is code, content is markdown) but adds boilerplate.

### 4. Future Screens for Vertical Slice

**Win/Death Screen** — static text with `[display]` fields (species, job, XL, turns, cause of death). litui can handle this today with existing features.

**Item Use Selection** — "Which potion to quaff?" shows a filterable inventory subset. Needs `[select]` from a dynamic list — litui can do this.

**Equipment Screen** — "Wearing: +2 leather armour. Wielding: +1 short sword." — `[display]` fields. litui can do this today.

### 5. Future DCSS Screens

**God Screen** — piety bar (progress!), ability list, conduct rules. Needs stateful `[progress]` and possibly `[foreach]` for ability list.

**Spell Screen** — spell list with failure rates, MP costs. `[foreach]` table. litui can do this today.

**Skill Screen** — skill levels with training sliders. Needs `[slider]` per skill inside `[foreach]` rows — `[foreach]` currently only supports read-only fields, not input widgets.

---

## Proposed Feature Set (Prioritized)

### Priority 1: Stateful `[progress]` — Unblocks Stat Panel

**Syntax**: `[progress](field_name)` where `field_name` is an `f64` on AppState (0.0 to 1.0).

**Optional config** for fill color:
```yaml
widgets:
  hp_bar:
    fill: "#8B0000"
  mp_bar:
    fill: "#1E1E64"
```
```markdown
[progress](hp_frac){hp_bar}
[progress](mp_frac){mp_bar}
```

**Generated code**: `ui.add(egui::ProgressBar::new(state.hp_frac as f32).fill(color))`

**Impact**: Unblocks HP/MP bars in stat panel. Also needed for XP bars, god piety bars, any gauge display.

### Priority 2: Conditional Styling via AppState — Unblocks Colored Text

**Syntax**: `[display](field){$style_field}` where `$` prefix means "read the style name from AppState" instead of using a literal style key.

```markdown
**HP:** [display](hp_text){$hp_style}
```

Where `state.hp_style: String` is set to `"hp_good"`, `"hp_warn"`, or `"hp_danger"` by game code, and those style names are defined in `_app.md` frontmatter.

**Generated code**:
```rust
let style = resolve_style(&state.hp_style, &styles);
ui.label(egui::RichText::new(&state.hp_text).color(style.color).size(style.size));
```

**Impact**: Colored HP/MP text, status effects with severity colors, item rarity colors, monster threat indicators.

### Priority 3: Streaming Append List — Unblocks Message Log

**Syntax**: `[log](field_name)` where `field_name` is a `Vec<String>` on AppState, rendered as a scroll area that sticks to the bottom.

```markdown
[log](messages)
```

**Generated code**:
```rust
egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
    for msg in &state.messages { ui.label(msg); }
});
```

**Impact**: Message log, combat log, chat systems. Different from `[foreach]` because it's a simple string list without structured row fields, and has scroll-to-bottom behavior.

### Priority 4: Conditional Sections — Nice to Have

**Syntax**: `[if](bool_field)` ... `[/if]()`

```markdown
[if](has_orb)
**ORB OF ZOT** {gold}
[/if]()
```

**Generated code**: `if state.has_orb { ... }`

**Impact**: Conditional UI sections (orb indicator, status warnings, equipment slots that may be empty). The empty-string `[display]` workaround covers simple cases, but conditional sections with styled text or multiple elements need this.

### Priority 5: Container Directives — Reduces Boilerplate

**Syntax** (frontmatter):
```yaml
page:
  name: Stats
  container: side_panel_right
  width: 180
```

Or for windows:
```yaml
page:
  name: Inventory
  container: window
  width: 350
  collapsible: false
```

**Impact**: Eliminates the Rust wrapper functions that create `egui::SidePanel`, `egui::Window`, `egui::CentralPanel`. The litui app could manage its own containers. Not critical — the 3-line Rust wrappers work fine — but would make litui a more complete solution.

### Priority 6: Input Widgets Inside [foreach] — Future Need

**Syntax**: `[slider]` or `[checkbox]` inside a `[foreach]` body.

```markdown
[foreach](skills)
| {name} | [slider]({level}) |
[/foreach]()
```

Where each row's slider modifies `state.skills[i].level`.

**Impact**: Skill training screen, spell memorization, keybinding configuration. Currently [foreach] is read-only. This would require generating mutable references to row fields and unique widget IDs per iteration.

---

## Summary Table

| # | Feature | Unblocks | Effort |
|---|---------|----------|--------|
| 1 | Stateful `[progress](field)` | Stat panel HP/MP bars, god piety, XP bar | Low |
| 2 | Conditional styling `{$field}` | Colored HP text, item rarity, threat colors | Medium |
| 3 | `[log](field)` append list | Message log, combat log | Low-Medium |
| 4 | `[if](bool)` conditional sections | Orb indicator, empty equipment slots | Medium |
| 5 | Container directives in frontmatter | Boilerplate reduction | Low |
| 6 | Input widgets in `[foreach]` | Skill training, spell memorize | High |

**Priority 1 alone** migrates the stat panel from 65 lines of egui to ~20 lines of markdown.
**Priorities 1-3** eliminate ALL hand-coded egui content rendering. egui would only provide structural containers.
**Priorities 1-4** achieve 100% litui for all game content with zero egui widget code.
