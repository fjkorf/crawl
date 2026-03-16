# litui Feature Proposal: Dynamic Data Rendering

## Background

We are building a Bevy roguelike game using litui for UI. litui works excellently for static content with injected values — our monster info popup uses `[display]` fields populated from ECS components each frame, and it renders beautifully from a markdown template. The Option C enhancement (`[display]` self-declaring String fields) was the key enabler.

However, two major screens had to fall back to hand-coded egui because they require **iterating over runtime-length collections** — something litui's compile-time markdown templates cannot express.

## The Two Screens That Couldn't Use litui

### 1. Character Creation — Selectable List from Runtime Data

The player chooses a species and a job from lists loaded from YAML files at startup. There are 27 species and 26 jobs. The screen shows two scrollable columns where one item in each column is highlighted as the current selection.

**What we wrote in egui:**
```rust
egui::ScrollArea::vertical().show(ui, |ui| {
    for (i, species) in species_list.iter().enumerate() {
        if ui.selectable_label(selected == i, &species.name).clicked() {
            selected = i;
        }
    }
});
```

**What we would have wanted to write in litui markdown:**
```markdown
# Choose Your Species

[select](selected_species){species_list}
```

Where `species_list` is a `Vec<SelectOption>` on AppState containing `{ label: String, description: String }` entries, and `selected_species` is a `usize` index tracking the current selection.

**Why litui couldn't do this:** The markdown template is compiled to a fixed set of egui calls at build time. There is no way to express "for each item in this Vec, render a selectable label." The number of items isn't known at compile time — it depends on how many YAML files exist on disk.

### 2. Inventory — Dynamic Item List

The player's inventory is a `Vec<ItemDef>` that changes constantly during gameplay (picking up items, dropping them, using potions). The screen shows a scrollable list with letter indices (a, b, c...) and item names.

**What we wrote in egui:**
```rust
for (i, item) in inventory.items.iter().enumerate() {
    let letter = (b'a' + i as u8) as char;
    ui.label(format!("{}) {}", letter, item.name));
}
```

**What we would have wanted to write in litui markdown:**
```markdown
# Inventory

**Gold:** [display](gold)

[foreach](items)
| {_index}) | {name} | x{quantity} |
[/foreach]
```

Where `items` is a `Vec<ItemRow>` on AppState and each `ItemRow` has `name: String` and `quantity: String` fields. The `{_index}` would be an auto-generated alphabetic index (a, b, c...).

**Why litui couldn't do this:** Same core issue — the template is static. An inventory can have 0 to 52 items. litui has no iteration construct.

## What Both Cases Have in Common

Both screens need to render a **variable-length list** from a **Vec on AppState**. The list contents change at runtime. Each item in the list has the same visual structure (a row with specific fields), but the number of rows is not known at compile time.

This is fundamentally different from litui's current model where every widget is declared once in markdown and maps to exactly one field on AppState.

## Proposed Solution: Collection Rendering Directives

### Option A: `[foreach]` Block Directive

A block-level directive that iterates over a Vec field on AppState:

```markdown
[foreach](items)
- **{name}** — {description}
[/foreach]
```

The macro would:
1. Recognize `items` as a `Vec<T>` field on AppState
2. Generate a loop in the render function: `for item in &state.items { ... }`
3. Inside the loop body, `{name}` and `{description}` resolve to fields on `T`
4. The struct `T` is auto-generated from the fields referenced inside the block

**AppState generation:**
```rust
pub struct ItemsRow {
    pub name: String,
    pub description: String,
}
// AppState gets:
pub items: Vec<ItemsRow>,
```

### Option B: `[select]` Widget for Selectable Lists

A specialized widget that renders a scrollable list with selection tracking:

```markdown
[select](selected_index){list_field}
```

Where `list_field` is a `Vec<String>` (labels) and `selected_index` is a `usize`. The macro generates:
```rust
egui::ScrollArea::vertical().show(ui, |ui| {
    for (i, label) in state.list_field.iter().enumerate() {
        if ui.selectable_label(state.selected_index == i, label).clicked() {
            state.selected_index = i;
        }
    }
});
```

**AppState generation:**
```rust
pub list_field: Vec<String>,
pub selected_index: usize,
```

### Option C: Both

`[foreach]` for general iteration (read-only display of dynamic lists), and `[select]` for interactive selection from dynamic lists. They serve different use cases:
- `[foreach]`: inventory display, log entries, spell lists, status effects
- `[select]`: species selection, job selection, god selection at altars, shop item purchase

## Concrete Markdown Examples

### Character Creation with `[select]`
```yaml
---
page:
  name: CharacterCreation
  label: Create Character
  default: true
widgets:
  confirm:
    track_hover: true
---

# Create Your Character

| | |
|---|---|
| [select](species_choice){species_names} | [select](job_choice){job_names} |

## [display](preview_name)

| Stat | Value |
|------|-------|
| Str | [display](preview_str) |
| Int | [display](preview_int) |
| Dex | [display](preview_dex) |

[button.primary](Start Game){confirm}
```

### Inventory with `[foreach]`
```yaml
---
page:
  name: Inventory
  label: Inventory
  default: true
---

# Inventory

**Gold:** [display](gold)

[foreach](items)
| {letter} | {name} | {quantity} |
[/foreach]
```

## Impact

These directives would make litui suitable for the majority of game UI screens. Currently, 15 of our 32 cataloged screens are good litui fits and 12 require hand-coded egui. With `[foreach]` and `[select]`, approximately 8 of those 12 would become litui-compatible, leaving only the dungeon grid, minimap, targeting overlay, and doll editor as egui-only.
