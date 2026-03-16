# [IMPLEMENTED] litui Feature Proposal: Display-Only Field Declarations

> **Status**: IMPLEMENTED. litui adopted Option C — `[display]` self-declares String fields. Working in monster info popup since MVP. This document is historical.

## Problem

litui's `[display](field)` widget reads a value from `AppState` and renders it as a label. However, the field must be *declared* by an input widget (`[slider]`, `[textedit]`, `[checkbox]`, etc.) somewhere in the app. There is no way to declare a field that is only ever written programmatically and displayed in markdown.

This makes litui unusable for a common UI pattern: **read-only data popups** where all content comes from application state (ECS components, database queries, API responses) and no user input widgets exist.

### Concrete Use Case

We're building a roguelike game in Bevy. When the player examines a monster, we show a stat card:

```markdown
# [display](monster_name)

| Stat | Value |
|------|-------|
| **HD** | [display](hd) |
| **HP** | [display](hp) |
| **AC** | [display](ac) |
| **EV** | [display](ev) |
| **Speed** | [display](speed) |
| **Attack** | [display](attack) |
```

Each field is a `String` populated from ECS components every frame:
```rust
state.monster_name = name.0.clone();
state.hp = format!("{}/{}", hp.current, hp.max);
```

This fails to compile because no widget declares `monster_name`, `hd`, etc.

## Proposed Solution

Add a `[data]` or `[field]` widget directive that declares a typed field on `AppState` without rendering any input control:

### Option A: Frontmatter Declaration

```yaml
---
fields:
  monster_name:
    type: string
  hd:
    type: string
  hp:
    type: string
---
```

This generates `pub monster_name: String`, `pub hd: String`, etc. on `AppState`, satisfying the `[display]` reference check.

### Option B: Inline Declaration

```markdown
[field](monster_name)
[field](hd)
```

A `[field]` directive that declares the field and generates the `AppState` member but renders nothing. Could also support a type parameter:

```markdown
[field.f64](temperature)
[field.bool](is_active)
```

### Option C: Display Self-Declares

Change `[display]` to both declare AND read the field, instead of requiring a separate declaration. This is the simplest change — `[display](foo)` would add `foo: String` to `AppState` if no other widget already declares it.

## Recommendation

**Option C** is the lowest-friction change. It matches user intent: "I want to show this value in the UI, and I'll set it from code." The current behavior — requiring a phantom `[textedit]` widget that's never rendered — is a workaround that adds noise.

If there's concern about type ambiguity (display fields are always String, but sliders are f64), the type could default to `String` for self-declared display fields, matching the fact that `[display]` always calls `format!` on the value anyway.

## Impact

This would unlock litui for:
- Game stat popups (our use case)
- Dashboard/monitor pages without corresponding form pages
- API response viewers
- Log/event detail panels
- Any UI where data flows in one direction: code → display
