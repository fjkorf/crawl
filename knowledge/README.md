# DCSS-to-Bevy Port — Knowledge Base

Research findings for porting Dungeon Crawl Stone Soup to the Bevy game engine with litui/egui UI.

## Architecture (Phase 1 — Complete)
- [global-state-graph.md](architecture/global-state-graph.md) — All 4 primary globals (env, you, crawl_state, Options) + 20 secondary globals mapped with fields and cross-references
- [turn-loop.md](architecture/turn-loop.md) — Complete turn flow from player input through world_reacts to monster turns, with Bevy schedule mapping
- [actor-trait-design.md](architecture/actor-trait-design.md) — Actor class analysis: 80 virtual methods, 75% non-shared. Recommends enum dispatch over dyn trait
- [adr-001-ecs-design.md](architecture/adr-001-ecs-design.md) — Architecture Decision Record: hybrid ECS (entities for monsters/items, resources for player/terrain)

## Subsystems (Phase 5 — Partial)
- [combat-system.md](subsystems/combat-system.md) — Attack phase system, damage pipeline, 21+ systems touched per attack. Cannot be pure function.

## Data Pipeline (Phase 3 — Complete)
- [asset-inventory.md](data-pipeline/asset-inventory.md) — All data sources: 3 YAML-ready (monsters, species, jobs), 5 need conversion (tiles, spells, features, DB, artifacts)

## Lua Integration (Phase 2 — Complete)
- [api-surface.md](lua-integration/api-surface.md) — 726 functions across 28 binding files, 230 critical for dungeon generation
- [des-file-format.md](lua-integration/des-file-format.md) — 70% custom DSL + 30% Lua. Parser is 6.5K lines. 197K lines of .des files to preserve.

## UI Mapping (Phase 4 — Complete)
- [screen-catalog.md](ui-mapping/screen-catalog.md) — 32 screens classified: 15 litui, 12 hand-coded egui, 5 hybrid. Main layout architecture defined.

## Translation Patterns (Phase 5 — Partial)
- [enum-catalog.md](translation-patterns/enum-catalog.md) — 15 core enums with ~2,700 total variants. monster_type largest at 1,075.

## Prototypes (Phase 7 — Not Started)
- (Pending implementation phase)
