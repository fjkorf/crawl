# litui Integration Issues and Questions

## Context

We're integrating litui into a Bevy roguelike game (DCSS port). We have three litui markdown pages — character creation (using `[select]`), inventory (using `[foreach]`), and monster info (using `[display]`). The monster info page works. The other two are new and hitting issues.

Our setup:
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

## Issue 1: `[foreach]` generates undefined symbols in `define_markdown_app!`

**Error:**
```
error[E0412]: cannot find type `__foreach__` in this scope
error[E0425]: cannot find value `__row` in this scope
```

**Our markdown** (`inventory.md`):
```markdown
---
page:
  name: Inventory
  label: Inventory
---

# Inventory

**Gold:** [display](inv_gold) {gold}

[foreach](inv_items)
| {letter} | {name} | {qty} |
|----------|--------|-------|
[/foreach]()

*Press i or Escape to close.*
```

**Observation:** The `[foreach]` tests in litui's own test suite all use `include_markdown_ui!`, not `define_markdown_app!`. The generated code references `__foreach__` and `__row` which appear to be internal symbols that don't resolve in the `define_markdown_app!` context.

**Question:** Does `[foreach]` work inside `define_markdown_app!`? Or is it currently only supported in `include_markdown_ui!`? If the latter, what would it take to support it in multi-page apps?

## Issue 2: `[select]` — same question

The `[select]` tests also use `include_markdown_ui!`. Our chargen.md uses `[select]` inside `define_markdown_app!`:

```markdown
| [select](chosen_species){species_list} | [select](chosen_job){job_list} |
```

**Question:** Does `[select]` work inside `define_markdown_app!`? The tests don't demonstrate this.

## Issue 3: Row type naming in `[foreach]`

When using `define_markdown_app!`, the generated row struct for `[foreach](inv_items)` needs to be accessible from outside the macro so we can populate it:

```rust
// We need to do this from game code:
let mut row = dcss_ui::pages::InvItemsRow::default();
row.letter = "a".to_string();
state.inv_items.push(row);
```

**Question:** What is the generated row struct name? Is it `{CollectionName}Row` (e.g., `InvItemsRow`)? Is it public? Can it be accessed from outside the module where `define_markdown_app!` is called?

## Issue 4: `[select]` inside table cells

Our character creation screen puts two `[select]` widgets side-by-side in a table:

```markdown
| Species | Background |
|---------|-----------|
| [select](chosen_species){species_list} | [select](chosen_job){job_list} |
```

**Question:** Is `[select]` supported inside table cells? The litui test fixture (`select_widget.md`) only shows `[select]` as a standalone block element.

## Issue 5: Button click tracking field name

For `[button.primary](Start_Game){start_game}` with `widgets: start_game: { track_hover: true }`, the button generates a click counter field.

**Question:** What is the exact field name? Is it `start_game_count`? `on_start_game_count`? How do we detect the button was clicked from Rust code?

## What Would Help

1. **A `define_markdown_app!` example that uses `[foreach]` and `[select]`** — the existing tests only demonstrate `include_markdown_ui!`. A multi-page app example would clarify whether these features work in the app context and show the correct patterns.

2. **Documentation of generated field/type names** — for `[foreach](items)`, what is the row type name, is it public, does it implement Default? For `[select](idx){list}`, what fields are generated on AppState?

3. **Clarification on widget-in-table support** — which widgets work inside GFM table cells? The widget-directives docs show standalone usage but don't specify table cell compatibility.
