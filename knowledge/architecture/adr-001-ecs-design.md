# ADR-001: ECS Architecture Design

## Status: PROPOSED (pending prototype validation)

## Context

DCSS uses ~4 global state objects (`env`, `you`, `crawl_state`, `Options`) with no encapsulation boundaries. All subsystems freely read/write all globals. We need to map this to Bevy's ECS.

## Decision

### Hybrid ECS: Entities for Monsters/Items, Resources for Player/Terrain

**Monsters → Bevy Entities** with components:
- `MonsterType`, `Health`, `Speed`, `Position`
- `AIState` (behaviour, foe, target, patrol)
- `MonsterEnchantments` (HashMap<EnchantType, MonEnchant>)
- `MonsterSpells`, `MonsterInventory`
- `GhostInfo` (optional)

**Items (floor) → Bevy Entities** with components:
- `ItemType`, `Position`, `ItemProperties`
- Pile linking via Bevy parent-child or a spatial index

**Player → Single Bevy Resource**:
- Too interconnected (~200+ fields) to decompose into components
- Accessed by nearly every system
- Singleton — no benefit from entity queries

**Terrain → Bevy Resource**:
- `TerrainGrid`: FixedArray equivalent for features, properties, colours
- `MonsterGrid`: FixedArray<Option<Entity>, GXM, GYM> for spatial lookup
- `ItemGrid`: FixedArray<Option<Entity>, GXM, GYM> for item piles
- Rarely changes, very frequently read

**Clouds, Traps, Shops → Bevy Entities** with Position component:
- Naturally spawn/despawn
- Queried spatially

**Game State → Bevy Resource**:
- `GameState` replacing `crawl_state`
- `GameOptions` replacing `Options` (read-only)

### Actor Polymorphism → Enum Dispatch

No `dyn Actor` trait. Instead:
```rust
enum ActorRef { Player, Monster(Entity) }
```
With free functions that match on the enum and dispatch to the appropriate data source.

**Rationale**: 75% of functionality is NOT shared. Callers immediately branch on type (747 is_player checks in codebase). True polymorphism is only used for constriction and a few shared calculations.

### Turn Loop → Phase-Based System Sets

```
Schedule::GameTurn {
    PreTurn          — housekeeping, revival, exp application
    PlayerInput      — wait for input (Bevy state + event)
    PlayerAction     — execute command
    WorldReaction    — sequential: {
        FinalEffects
        EnvironmentEffects
        PlayerReacts
        MonsterTurns   — iterative priority queue (NOT parallelizable)
        PostMonster
        TimeAdvance
        CloudManage
        ViewUpdate
    }
    TurnEnd          — counter, save check
}
```

**Rationale**: Monster turns must be sequential (priority queue with re-queuing). The entire WorldReaction phase is sequential because monsters can affect each other during their turns.

### Combat → Direct Port (Phase 1), Command Pattern (Phase 2)

Phase 1: Monolithic system functions with many SystemParams, directly porting C++ logic.
Phase 2: Refactor to return `Vec<CombatEffect>` for testability.

**Rationale**: Combat touches 21+ subsystems and is recursive (cleave, riposte). A clean functional design requires significant restructuring. Direct port first, refactor later.

## Consequences

### Positive
- Monsters/items benefit from Bevy queries for rendering, spatial lookups, LOS
- Player as Resource avoids the overhead of entity queries for a singleton
- Terrain as Resource matches its access pattern (constant reads, rare writes)
- Enum dispatch is explicit and Rust-idiomatic (no trait objects, no vtables)
- Phase-based scheduling maps naturally to the existing turn structure

### Negative
- MonsterGrid Resource must be manually synchronized with entity positions
- Cross-references between monsters/items still need Entity IDs instead of array indices
- Combat's direct port will be hard to test in isolation
- Monster turns cannot leverage Bevy's parallel scheduling

### Risks
- MonsterGrid sync bugs (entity position vs grid position mismatch)
- Entity ID stability across save/load cycles
- Player Resource becoming a god object

## Validation

Build a prototype (Phase 1.2 from research plan):
1. 500 monster entities on a grid with Position + Speed components
2. MonsterGrid resource for spatial lookups
3. Sequential turn processing via priority queue
4. Compare performance/ergonomics with array-based Resource approach
