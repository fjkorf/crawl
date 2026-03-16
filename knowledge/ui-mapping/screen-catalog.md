# DCSS UI Screen Catalog — litui/egui Mapping

## Widget System (ui.h)

Core widgets: Bin, Box (flexbox), Grid, Stack, Switcher, Scroller, Text, Image, Checkbox, TextEntry, Dungeon, PlayerDoll, Popup

## Tile Regions (14 panels)

| Region | Type | Purpose |
|--------|------|---------|
| DungeonRegion | Grid/Interactive | Main game view — tiles + targeting |
| StatRegion | Text | HP, MP, AC, EV, stats display |
| MessageRegion | Text | Game message log |
| MapRegion | Specialized | Minimap |
| InventoryRegion | Grid | Items with quantities and keys |
| SpellRegion | Grid | Memorized spells |
| MemoriseRegion | Grid | Available spells to learn |
| AbilityRegion | Grid | Player abilities |
| SkillRegion | Grid | Skill levels and training |
| MonsterRegion | Grid | Visible monsters |
| TabbedRegion | Container | Tab switching for inventory categories |
| TextRegion | Text | CRT-style output |
| CRTRegion | Text | Highlightable text menu |
| DollEditRegion | Specialized | Character appearance |

## Screen Classification

### Strong Fit for litui (text-heavy, semi-static) — 15 screens

| Screen | Data Source | Interactivity |
|--------|-----------|---------------|
| Item Description | item_def → text | Read-only |
| Spell Description | spell_data → text | Read-only |
| Monster Description | monster_info → text | Read-only |
| God Description | god data → text | Read-only |
| Ability Description | ability data → text | Read-only |
| Skill Description | skill data → text | Read-only |
| Mutation Description | mutation data → text | Read-only |
| Feature Description | feature DB → text | Read-only |
| Character Dump | player state → text | Read-only |
| Help Screens | static text | Read-only |
| High Scores | score data → text | Read-only, scrollable |
| Overview Screen | dungeon map data → text | Read-only |
| Runes Screen | collected runes → text | Read-only |
| Known Objects | ID database → text | Read-only, filterable |
| Resistances Screen | player resists → text | Read-only |

**litui approach**: Each screen gets a `.md` file with frontmatter. Dynamic data injected via state bindings.

### Hand-Coded egui (interactive, dynamic) — 12 screens

| Screen | Why Not litui | egui Approach |
|--------|--------------|---------------|
| Dungeon View | Real-time tile rendering | Bevy 2D renderer (NOT egui) |
| Minimap | Pixel-level rendering | Bevy 2D or egui Canvas |
| Direction Chooser | Animated overlay on dungeon | Bevy overlay system |
| Stat Panel | Real-time updating | **litui [progress] + ::: style + ::: if** (migrated) |
| Message Log | Scrolling, styled text | **litui [log]** (migrated) |
| Inventory Grid | Icons + hotkeys + quantity | **litui [foreach]** (migrated from egui) |
| Spell Grid | Icons + failure rates | egui Grid with icons |
| Ability Grid | Icons + costs + failure | egui Grid with icons |
| Skill Training | Real-time XP allocation | egui Grid with sliders |
| Shop Interface | Prices, gold, buy/sell | egui Window with lists |
| Character Creation | Multi-step, validation | **litui [select]** (migrated from egui) |
| Doll Editor | Visual customization | egui or Bevy 2D |

### Hybrid (litui descriptions + egui interaction) — 5 screens

| Screen | litui Part | egui Part |
|--------|-----------|-----------|
| Inventory Menu | Item descriptions | List selection, hotkeys |
| Spell Menu | Spell descriptions | Selection, memorization |
| Religion Screen | God lore, ability text | Piety bar, ability activation |
| Mutation Screen | Mutation descriptions | Cure potion interaction |
| Stash Search | Item descriptions | Search input, filtering |

## Layout Architecture

### Main Game Layout (Bevy + egui hybrid)

```
┌─────────────────────────────────────────────┐
│ ┌───────────────────────┐ ┌───────────────┐ │
│ │                       │ │  Stat Panel   │ │
│ │                       │ │  (egui)       │ │
│ │   Dungeon View        │ │  HP/MP/AC/EV  │ │
│ │   (Bevy 2D)           │ │  Status icons │ │
│ │                       │ ├───────────────┤ │
│ │                       │ │  Monster List │ │
│ │                       │ │  (egui)       │ │
│ ├───────────────────────┤ ├───────────────┤ │
│ │   Message Log         │ │  Minimap      │ │
│ │   (egui ScrollArea)   │ │  (Bevy 2D)   │ │
│ └───────────────────────┘ └───────────────┘ │
└─────────────────────────────────────────────┘
```

### Overlay Screens (egui Windows)
- Inventory, Spells, Abilities → egui::Window overlaying dungeon
- Description popups → litui-rendered content in egui::Window
- Character creation → full-screen egui

## litui Template Estimates

- **15 static screens** → 15 markdown files
- **5 hybrid screens** → 5 markdown files + egui integration code
- Each markdown file: ~50-200 lines with YAML frontmatter for styles/widgets
- Total litui content: ~1,500-3,000 lines of markdown

## Key litui Features Needed

1. **Dynamic text injection**: Monster stats, item properties from game state
2. **Conditional sections**: Show/hide based on identification status, god, form
3. **Styled text**: Color-coded damage types, resistance levels
4. **Scrollable content**: Long descriptions, spell lists
5. **Multi-page**: `define_markdown_app!` for character creation wizard

## Priority for Vertical Slice

1. Dungeon View (Bevy 2D) — core gameplay
2. Stat Panel (egui) — HP/MP display
3. Message Log (egui) — game messages
4. Monster Info (litui) — targeting/examining
5. Inventory (egui) — item management
6. Everything else is post-vertical-slice
