# Actor Trait Design for Rust/Bevy Port

## C++ Actor Hierarchy Summary

`actor` is the base class for both `player` and `monster`, with ~80 virtual methods.

### Method Distribution
- **Pure virtual (subclass must implement)**: ~55 methods
- **Default implementation in actor.cc**: ~25 methods (constriction, movement, equipment scanning, shield logic)
- **Monster overrides**: 184 methods
- **Player overrides**: 192 methods
- **Monster-unique methods**: ~150 (enchantments, AI, ghosts, inventory)
- **Player-unique methods**: ~200 (progression, religion, spells, durations)
- **Shared via actor**: ~60 methods

### Key Finding: Explicit Type Branching, Not True Polymorphism
- `is_player()`/`is_monster()` calls: 747 occurrences across 54 files
- `as_player()`/`as_monster()` casts: 452 occurrences across 54 files
- Zero `dynamic_cast<>` usage — all casting is explicit
- 88 of 798 source files (~11%) use `actor*` polymorphically
- **Pattern**: Code gets `actor*`, immediately checks type and branches

### Virtual Method Categories

| Category | Count | Examples |
|----------|-------|---------|
| Type/Identity | 5 | `is_player()`, `mindex()`, `as_monster()` |
| Movement | 6 | `move_to()`, `blink_to()`, `teleport()` |
| Basic Attributes | 7 | `get_hit_dice()`, `body_size()`, `deity()` |
| Equipment | 16 | `weapon()`, `shield()`, `wearing()`, `attack_delay()` |
| Combat | 3 | `attacking()`, `fumbles_attack()`, `can_throw_large_rocks()` |
| Damage/Healing | 10 | `hurt()`, `heal()`, `poison()`, `paralyse()` |
| Resistances | 18 | `res_fire()`, `res_cold()`, `willpower()`, `res_torment()` |
| Status/Conditions | 10 | `paralysed()`, `confused()`, `asleep()`, `helpless()` |
| Visibility | 7 | `invisible()`, `can_see_invisible()`, `visible_to()` |
| Holiness/Nature | 12 | `holiness()`, `is_holy()`, `has_blood()`, `is_stationary()` |
| Constriction | 10+ | `start_constricting()`, `attempt_escape()`, `constriction_damage()` |
| Terrain | 5 | `can_pass_through_feat()`, `is_habitable_feat()` |
| Naming | 5 | `name()`, `pronoun()`, `conj_verb()` |
| Auras | 8 | `halo_radius()`, `silence_radius()`, `umbra_radius()` |
| Buffs/Traits | 14 | `skill()`, `stealth()`, `clarity()`, `faith()` |

### Actor Iteration System (act-iter.h/cc)
- `actor_near_iterator`: Iterates player + monsters in LOS. Returns `actor*`.
- `monster_near_iterator`: Only monsters in LOS.
- `monster_iterator`: All alive monsters on level.
- All return bare pointers — no ownership.

## Recommended Rust/Bevy Design

### Do NOT use `dyn Actor` trait

**Reasons:**
1. ~75% of functionality is NOT shared between player and monster
2. Callers immediately branch on type after getting `actor*` (747 is_player checks)
3. Player is a singleton resource, not an entity
4. Monster and player data layouts are completely different

### Recommended: Component-Based with Enum Dispatch

```rust
// Shared stats as a component (for monsters) / embedded in resource (for player)
pub struct ActorStats {
    pub hp: i32,
    pub max_hp: i32,
    pub position: Coord,
    pub mid: MonsterId,
}

// For functions that need to operate on "any actor"
pub enum ActorRef {
    Player,
    Monster(Entity),
}

// Resist queries — shared logic, different data sources
pub fn res_fire(actor: ActorRef, world: &World) -> i32 {
    match actor {
        ActorRef::Player => { /* read from Player resource */ }
        ActorRef::Monster(e) => { /* query Monster components */ }
    }
}

// Constriction — the one truly shared system — as components
#[derive(Component)]
pub struct ConstrictionState {
    pub constricted_by: Option<ActorRef>,
    pub constricting: Vec<ActorRef>,
    pub escape_attempts: i32,
}
```

### Monster Components (Bevy entities)
```rust
#[derive(Component)] pub struct MonsterType(monster_type);
#[derive(Component)] pub struct Health { pub hp: i32, pub max_hp: i32 }
#[derive(Component)] pub struct Speed { pub base: i32, pub increment: i32 }
#[derive(Component)] pub struct AIState { pub behaviour: BehType, pub foe: Option<Entity>, pub target: Coord }
#[derive(Component)] pub struct MonsterEnchantments(HashMap<EnchantType, MonEnchant>);
#[derive(Component)] pub struct MonsterSpells(Vec<MonSpellSlot>);
#[derive(Component)] pub struct MonsterInventory([Option<Entity>; NUM_MONSTER_SLOTS]);
```

### Player as Resource
```rust
#[derive(Resource)]
pub struct Player {
    pub stats: ActorStats,
    pub species: SpeciesType,
    pub job: JobType,
    pub skills: [u8; NUM_SKILLS],
    pub spells: [SpellType; MAX_KNOWN_SPELLS],
    pub duration: [i32; NUM_DURATIONS],
    pub mutation: [u8; NUM_MUTATIONS],
    pub religion: GodType,
    pub piety: u8,
    pub inv: Vec<ItemDef>,
    // ... ~200 more fields
}
```

### Iteration Replacement
```rust
// actor_near_iterator → two separate queries + player check
fn actors_near(pos: Coord, los: LosType,
               monsters: &Query<(Entity, &ActorStats), With<MonsterType>>,
               player: &Res<Player>) -> Vec<ActorRef> {
    let mut result = Vec::new();
    if player.stats.position.in_los(pos, los) {
        result.push(ActorRef::Player);
    }
    for (entity, stats) in monsters.iter() {
        if stats.position.in_los(pos, los) {
            result.push(ActorRef::Monster(entity));
        }
    }
    result
}
```
