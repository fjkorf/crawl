# Engine / Middleware / Content Architecture

## Definitions

**Engine** — Systems that are game-agnostic or structurally foundational. They process data but don't define what the data IS. An engine system works regardless of whether the game has 7 monsters or 1,075. Changing engine code changes HOW the game works.

**Content** — Data and definitions that describe WHAT exists in the game. Monster stats, spell effects, vault layouts, species aptitudes, item properties. Content can be added, removed, or modified without changing engine systems. Ideally loaded from data files, not compiled into code.

**Middleware** — Translation layers that bridge engine and content. The .des parser, Lua runtime, YAML loaders, enum-to-data mappings, tile lookup tables. Middleware makes content consumable by the engine. It knows the shape of both sides but belongs to neither.

## Current State

### Engine (crates/dcss_game, crates/dcss_core)
| System | Status | Remaining |
|--------|--------|-----------|
| ECS architecture (Bevy plugin) | Done | — |
| Turn loop (input → action → world reaction) | Done | — |
| Terrain grid + sprite rendering | Done | — |
| Player movement + collision | Done | — |
| Camera follow + projection | Done | — |
| Melee combat resolution | Done | Ranged combat, multi-attack, brands |
| Monster AI (chase + attack) | Done | Pathfinding (A*), spell casting, fleeing, abilities |
| Monster death + despawn | Done | XP award on kill |
| Door interaction | Done | Open closed doors |
| Stairs / level transition | Done | Up/down, 5-level dungeon, LevelStore persistence |
| Floor item spawning + auto-pickup | Done | Random items per level, auto-pickup on walk |
| Item use system | Done | Quaff potions (heal), wield weapons, wear armour |
| Equipment slots | Done | weapon + armour on Player, affects damage/AC |
| FOV / line of sight | Done | Recursive shadowcasting in fov.rs, 8-tile radius |
| Experience + leveling | Done | HD*10 XP per kill, threshold-based level-up, HP/MP increase |
| Win condition (Orb) | Done | Orb on D:5, has_orb flag, ascend to surface = win |
| Level persistence | Done | LevelStore saves/restores grid + monsters + items |
| Status effects / durations | Not started | Duration array on Player, tick-down per turn |
| Save / load (to disk) | Not started | Serialize full game state to file |
| God system framework | Not started | Piety resource, prayer, conduct tracking |
| Spell system framework | Not started | Spell list, MP cost, targeting, effects |
| Death / morgue | Not started | Character dump, score recording |

### Middleware (crates/dcss_lua, crates/dcss_tiles)
| System | Status | Remaining |
|--------|--------|-----------|
| .des file parser | Done (5,996 vaults, 0 errors) | — |
| Lua execution pipeline (mlua) | Done (75% vault success) | Remaining 25% need more API stubs |
| SUBST/NSUBST/SHUFFLE processing | Done | — |
| Vault → TerrainGrid conversion | Done | Monster/item placement from vault data |
| Monster YAML loader | Done (7 monsters) | Load ALL 672 monster files |
| Species YAML loader | Done (27 species) | — |
| Job YAML loader | Done (26 jobs) | — |
| Tile registry (individual PNGs) | Done (15 tiles) | Tile atlas (packed sprite sheets), more variants |
| C++ spell data extraction | Not started | Extract spl-data.h → YAML/RON |
| C++ mutation data extraction | Not started | Extract mutation-data.h → YAML/RON |
| Database file parser (monspeak, randname) | Not started | Custom DSL with template expansion |
| Artifact data parser | Not started | art-data.txt → structured data |
| DungeonFeatureType enum (175 variants) | Done | Wire to terrain rendering |
| MonsterType enum (30 subset) | Done | Expand to full 1,075 |
| Remaining 13 C++ enums | Not started | spell_type, duration_type, god_type, etc. |
| Lua dgn.* API bindings | Partial (~45 stubs) | Full 230 functions |
| Lua crawl.*/you.* API bindings | Minimal stubs | Full implementation tied to engine state |
| Lua library loading (vault.lua, lm_*.lua) | Partial | Full loading with all dependencies |
| Level layout algorithms | Not started | Room generation, corridor connection, vault placement |

### Content (dat/des/*, dat/mons/*, dat/species/*, dat/jobs/*)
| Content | Status | Remaining |
|---------|--------|-----------|
| Monster definitions (672 YAML files) | 7 loaded | Load all, wire MonsterType enum |
| Species definitions (27 YAML files) | All loaded | Wire to gameplay (aptitudes, mutations, abilities) |
| Job definitions (26 YAML files) | All loaded | Wire to gameplay (starting equipment, skills) |
| Vault definitions (5,996 in 143 .des files) | Parseable, 75% executable | Level generation using vaults |
| Spell definitions (spl-data.h, 514 spells) | Not extracted | Need extraction tool + data format |
| God definitions | Not started | Abilities, conduct rules, piety thresholds |
| Item definitions (brands, egos, unrandarts) | Not started | art-data.txt, item-prop tables |
| Mutation definitions (mutation-data.h) | Not started | Need extraction |
| Branch definitions (branch.h, 44 branches) | Not started | Depth, monster sets, features |
| Tile assets (8,417 PNGs) | 15 loaded | Full tile set loading |
| Description databases (monspeak, help, etc.) | Not started | Template-expanded text |

## Dependency Flow

```
Content (data files)
    ↓ loaded by
Middleware (parsers, loaders, Lua runtime)
    ↓ produces
Engine-consumable structs (MapDef, MonsterDef, SpeciesDef, ItemDef)
    ↓ consumed by
Engine (ECS systems, Bevy rendering, game logic)
    ↓ rendered via
UI (litui markdown, egui panels, Bevy 2D sprites)
```

## Priority by Layer (Updated — Vertical Slice Complete)

### Engine priorities (next features)
1. ~~FOV~~ ✅ | ~~Item use~~ ✅ | ~~Equipment~~ ✅ | ~~XP/leveling~~ ✅
2. Ranged combat — not just melee
3. Status effects / durations — poison, haste, etc.
4. Save/load to disk — persistence across sessions
5. Spell system framework — MP cost, targeting, effects

### Middleware priorities (what makes content accessible)
1. Load all 672 monster YAML files — full bestiary
2. Extract spell data from C++ headers — spell system needs data
3. Level layout algorithm — connect multiple vaults into a level
4. Expand Lua API stubs — push vault execution past 90%

### Content priorities (what makes it DCSS)
1. D:1-15 branch with appropriate monster sets per depth
2. Starting equipment from job definitions
3. God altars and basic worship
4. Shops
5. Rune branches
