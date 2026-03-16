# DCSS-to-Bevy Port — Knowledge Base

Research and implementation findings for porting Dungeon Crawl Stone Soup to Bevy 0.18 with litui/egui UI.

## Architecture (Phase 1 — Complete)
- [global-state-graph.md](architecture/global-state-graph.md) — All 4 primary globals (env, you, crawl_state, Options) + 20 secondary globals mapped
- [turn-loop.md](architecture/turn-loop.md) — Turn flow from player input through world_reacts to monster turns, with Bevy schedule mapping
- [actor-trait-design.md](architecture/actor-trait-design.md) — Actor class analysis: 80 virtual methods, 75% non-shared. Enum dispatch, not dyn trait
- [adr-001-ecs-design.md](architecture/adr-001-ecs-design.md) — ADR: hybrid ECS (entities for monsters/items, resources for player/terrain). **VALIDATED** via MVP.

## Subsystems (Partial)
- [combat-system.md](subsystems/combat-system.md) — C++ attack phase system analysis. Simplified Rust version implemented in dcss_core/combat.rs.

## Data Pipeline (Complete)
- [asset-inventory.md](data-pipeline/asset-inventory.md) — All data sources: 3 YAML-ready (monsters, species, jobs — all loading), tiles (loading individual PNGs), 5 need conversion

## Lua Integration (Phases A-D Complete)
- [api-surface.md](lua-integration/api-surface.md) — 726 functions across 28 binding files, 230 critical for dungeon generation
- [des-file-format.md](lua-integration/des-file-format.md) — 70% custom DSL + 30% Lua. Parser is 6.5K lines. 197K lines of .des files.
- [parser-internals.md](lua-integration/parser-internals.md) — Flex+Bison architecture. Ported as hand-written Rust parser.
- [parser-port.md](lua-integration/parser-port.md) — 5,996 vaults parsed, 75% execute, end-to-end vault rendering working
- [mlua-poc.md](lua-integration/mlua-poc.md) — mlua 0.10 API patterns, dual Lua states, userdata

## UI Mapping (Complete)
- [screen-catalog.md](ui-mapping/screen-catalog.md) — 32 screens classified. Chargen and inventory migrated to litui. See litui-integration.md for current status.

## Translation Patterns (Partial)
- [enum-catalog.md](translation-patterns/enum-catalog.md) — 15 core enums (~2,700 variants). DungeonFeatureType (175) and MonsterType (30 subset) ported to Rust.

## Prototypes (Active)
- [mvp-findings.md](prototypes/mvp-findings.md) — Bevy 0.18 API patterns, screenshot testing, feature flags
- [litui-integration.md](prototypes/litui-integration.md) — **Current status**: chargen [select], inventory [foreach], monster [display] all working
- [medium-term-plan.md](prototypes/medium-term-plan.md) — Prioritized roadmap for remaining features

## Historical (Implemented Proposals)
- [litui-display-only-proposal.md](prototypes/litui-display-only-proposal.md) — **IMPLEMENTED**: Option C (display self-declares fields)
- [litui-dynamic-data-proposal.md](prototypes/litui-dynamic-data-proposal.md) — **IMPLEMENTED**: [foreach] and [select] directives
- [litui-integration-issues.md](prototypes/litui-integration-issues.md) — **RESOLVED**: All 7 issues addressed by litui
