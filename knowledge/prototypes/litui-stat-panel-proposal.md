# litui: Why the Stat Panel Can't Use litui (and What Would Fix It)

## The Screen

The stat panel is a right-side panel showing player vitals: HP bar, MP bar, AC, EV, stats, XL, XP, gold, and a conditional "ORB OF ZOT" indicator. It updates every frame.

## What We Wrote in egui (60 lines)

```rust
let hp_frac = player.hp as f32 / player.max_hp.max(1) as f32;
ui.horizontal(|ui| {
    ui.label("HP:");
    let color = if hp_frac > 0.5 { GREEN } else if hp_frac > 0.25 { YELLOW } else { RED };
    ui.colored_label(color, format!("{}/{}", player.hp, player.max_hp));
});
ui.add(egui::ProgressBar::new(hp_frac).fill(egui::Color32::DARK_RED));
```

## What We'd Want to Write in litui

```markdown
**HP:** [display](hp_text) {hp_color}
[progress](hp_frac)

**MP:** [display](mp_text) {mp_color}
[progress](mp_frac)

| | |
|---|---|
| **AC** | [display](ac) |
| **EV** | [display](ev) |
| **XL** | [display](xl) |
| **XP** | [display](xp_text) |
| **Gold** | [display](gold) |

[display](orb_text) {orb_style}
```

## Three Blockers

### 1. `[progress]` Is Stateless — Cannot Read From AppState

`[progress](0.75)` renders a fixed bar. There's no `[progress](hp_frac)` that reads a field. We need `[progress]` to work like `[slider]` — accept a field name and read from AppState.

**Current**: `[progress](0.75)` → hardcoded f32 literal
**Needed**: `[progress](hp_frac)` → reads `state.hp_frac: f64`

The implementation would be similar to `[slider]`: add a `WidgetField::Stateful { name, ty: WidgetType::F64 }` for progress fields, and generate `ui.add(egui::ProgressBar::new(state.hp_frac as f32))` instead of a literal.

### 2. No Conditional Styling — Colors Can't Depend on Runtime State

HP text should be green when above 50%, yellow 25-50%, red below 25%. litui styles are compile-time constants defined in YAML frontmatter. There's no way to switch styles based on a runtime value.

**Current**: `{hp_color}` would be a single static style
**Needed**: Conditional style selection from AppState, e.g.:
```markdown
[display](hp_text) {hp_style}
```
Where `hp_style` is a `String` field on AppState that the Rust code sets to `"hp_good"`, `"hp_warn"`, or `"hp_danger"` each frame, and litui resolves it to the corresponding style definition.

This is a more complex feature request — it requires the style key to be a runtime value rather than a compile-time constant. One approach: a `[styled_display]` widget that takes both a value field and a style field:
```markdown
[styled_display](hp_text){hp_style_name}
```
Where both `hp_text: String` and `hp_style_name: String` are on AppState, and the generated code looks up the style by name at runtime.

### 3. No Conditional Visibility — "ORB OF ZOT" Shows Only When Player Has It

The orb indicator should only appear when `player.has_orb` is true. litui markdown is always rendered — there's no `[if](condition)` directive to conditionally include content.

**Current**: No way to express "show this line only when a bool field is true"
**Needed**: Either:
- `[if](has_orb)` ... `[/if]()` block directive (like `[foreach]` but for conditional sections)
- Or a `[display]` that renders nothing when the value is empty (already works — set `orb_text = ""` when no orb)

The empty-string workaround actually handles this case: set `state.orb_text = ""` when no orb, `state.orb_text = "ORB OF ZOT"` when player has it. The conditional visibility is solved by convention. But conditional STYLING (the gold color) still requires blocker #2.

## Summary

| Blocker | Feature Needed | Difficulty Estimate |
|---------|---------------|-------------------|
| Stateful `[progress]` | `[progress](field)` reads f64 from AppState | Low — same pattern as `[slider]`, minus the input handling |
| Conditional styling | Style key from AppState field, resolved at runtime | Medium — requires runtime style lookup instead of compile-time |
| Conditional visibility | `[if](bool_field)` block | Medium — but workaround exists (empty string) |

**If litui added stateful `[progress]`**, the stat panel could be ~80% litui. The stat table and XP/gold display are straightforward `[display]` fields. Only the conditional HP/MP coloring would remain as egui.

**If litui also added conditional styling**, the stat panel could be 100% litui — the entire screen expressed in ~25 lines of markdown instead of 60 lines of Rust.
