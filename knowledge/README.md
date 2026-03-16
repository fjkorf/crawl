# DCSS-to-Bevy Port — Knowledge Base

Porting Dungeon Crawl Stone Soup to Bevy 0.18 with litui/egui UI.

**Current state**: Vertical slice complete and playable. 37 tests passing. 7 crates. 100% litui for game UI content (5 pages). Full gameplay loop: create character → explore 5-level dungeon with FOV → fight monsters (XP/leveling) → collect items (potions/weapons/armour) → find Orb on D:5 → ascend → win.

## Architecture (Complete)
- [global-state-graph.md](architecture/global-state-graph.md) — C++ globals mapped to ECS (reference)
- [turn-loop.md](architecture/turn-loop.md) — Turn flow with Bevy schedule mapping
- [actor-trait-design.md](architecture/actor-trait-design.md) — Enum dispatch, not dyn trait
- [adr-001-ecs-design.md](architecture/adr-001-ecs-design.md) — Hybrid ECS. **VALIDATED**.
- [engine-content-middleware.md](architecture/engine-content-middleware.md) — Layer separation and remaining work

## Subsystems
- [combat-system.md](subsystems/combat-system.md) — Simplified Rust combat in dcss_core/combat.rs

## Data Pipeline
- [asset-inventory.md](data-pipeline/asset-inventory.md) — Data sources: YAML (loading), tiles (loading), C++ tables (pending extraction)

## Lua Integration (Phases A-D Complete)
- [api-surface.md](lua-integration/api-surface.md) — 726 Lua functions cataloged
- [des-file-format.md](lua-integration/des-file-format.md) — .des format spec
- [parser-internals.md](lua-integration/parser-internals.md) — Flex+Bison architecture (reference)
- [parser-port.md](lua-integration/parser-port.md) — 5,996 vaults parsed, 75% execute
- [mlua-poc.md](lua-integration/mlua-poc.md) — mlua API patterns

## UI Mapping
- [screen-catalog.md](ui-mapping/screen-catalog.md) — 32 DCSS screens classified

## Translation Patterns
- [enum-catalog.md](translation-patterns/enum-catalog.md) — 15 enums (~2,700 variants). 2 ported.

## Prototypes (Active)
- [litui-integration.md](prototypes/litui-integration.md) — 100% litui: 5 pages with :: syntax, [select], [foreach], [display], [progress], [log], ::: if/style
- [mvp-findings.md](prototypes/mvp-findings.md) — Bevy 0.18 patterns, screenshot testing
- [litui-stat-panel-proposal.md](prototypes/litui-stat-panel-proposal.md) — 6 features for full UI coverage (partially implemented)

## Historical (Implemented)
- [litui-display-only-proposal.md](prototypes/litui-display-only-proposal.md) — **IMPLEMENTED**: Option C
- [litui-dynamic-data-proposal.md](prototypes/litui-dynamic-data-proposal.md) — **IMPLEMENTED**: [foreach] + [select]
- [litui-integration-issues.md](prototypes/litui-integration-issues.md) — **RESOLVED**: All 7 issues
- [medium-term-plan.md](prototypes/medium-term-plan.md) — **COMPLETED**: All items done
