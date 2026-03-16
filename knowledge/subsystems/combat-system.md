# DCSS Combat System Analysis

## Architecture

Combat uses a phase-based OOP design with class hierarchy:
- `attack` (base) → `melee_attack` → specific attack resolution
- Also: `ranged_attack` (not deeply analyzed yet)

Key files:
- `attack.h/cc` — base attack class
- `melee-attack.h/cc` — melee specialization (~5.4K lines)
- `fight.h/cc` — top-level dispatch
- `beam.h/cc` — projectiles, area effects

## Attack Phase Sequence

```
attack()
├── handle_phase_attempted()  — reach/constriction/momentum checks
├── test_hit()                — to-hit vs evasion
├── handle_phase_blocked()    — shield block resolution
├── handle_phase_hit()        — damage calculation + brand effects
│   ├── calc_damage()         — THE complex function
│   ├── player_monattk_hit_effects()
│   ├── check_unrand_effects()
│   └── handle_phase_damaged()  — apply damage, check death
├── handle_phase_dodged()     — retaliation triggers
├── handle_phase_aux()        — auxiliary unarmed attacks
├── handle_phase_end()        — cleaving, multi-hit, fugue
└── handle_phase_killed()     — monster_die() with kill credit
```

## Damage Calculation Pipeline (calc_damage)

1. **Base damage**: weapon_damage() or calc_base_unarmed_damage()
2. **Skill modifiers**: fighting skill, weapon skill, slaying
3. **AC application**: defender AC, vulnerability stacking
4. **Brand effects**: elemental, reaping, drain, chaos
5. **Final multipliers**: stab bonus, martial forms, cleave penalty

## Global State Dependencies

### Reads From (~20+ sources)
- `you.skill()`, `you.dex()`, `you.experience_level`, `you.slaying()`
- `you.duration[]` (DUR_VERTIGO, DUR_BLIND, DUR_CONFUSING_TOUCH, DUR_DEVIOUS, etc.)
- `you.form` (transformation bonuses)
- `you.mutation[]` (MUT_REFLEXIVE_HEADBUTT, MUT_BLACK_MARK, etc.)
- Equipment properties, unrand effects
- `defender->evasion()`, `defender->apply_ac()`, `defender->res_*()`, `defender->has_ench()`
- `env.grid()` (terrain), `crawl_state` (game mode flags)

### Writes To (~21 systems)
1. Defender HP (via `hurt()`)
2. Monster enchantments (via `add_ench()`)
3. Monster behavior (via `behaviour_event()`)
4. Player skills (via `practise_hitting()`)
5. Player durations (DUR_CONFUSING_TOUCH, DUR_DEVIOUS, etc.)
6. Player attributes (ATTR_SUNDERING_CHARGE)
7. God conduct (DID_EVIL, DID_CHAOS, etc.)
8. Message queue
9. Animation queue
10. Fineff queue (blood, blinks, curses, detonations)
11. Monster death processing (XP, loot, drops)
12. Pet target (`you.pet_target`)
13. Retaliation triggers (riposte, headbutt, executioner — **recursive**)
14. Dithmenos shadow attacks
15. Paragon attack triggers
16. Solar ember blasts
17. Medusa stinger attacks
18. Warmup strikes / rev tracking
19. Berserk penalty
20. Action counting (stats/hints)
21. Hydra head tracking

## Can Combat Be Made Pure?

**No — deeply entangled with global state.** Key blockers:

1. **Recursive attacks**: Cleaving, riposte, headbutt spawn NEW `melee_attack` instances
2. **Skill training**: Permanently modifies player progression during attack
3. **God conduct**: Must notify god system of moral implications
4. **Monster death**: Full death processing (XP, drops, effects) during attack
5. **Messaging**: Interleaved with calculation (can't defer easily)

### Recommended Port Strategy: Command Pattern

```rust
struct AttackResult {
    hit: bool,
    damage: i32,
    effects: Vec<CombatEffect>,
    messages: Vec<GameMessage>,
    followup_attacks: Vec<AttackRequest>,
}

enum CombatEffect {
    DamageDealt { target: ActorRef, amount: i32, type_: BeamType },
    EnchantmentApplied { target: ActorRef, ench: EnchantType, duration: i32 },
    SkillPractice { skill: SkillType, amount: i32 },
    GodConduct { conduct: ConductType, level: i32 },
    MonsterDied { entity: Entity, killer: ActorRef },
    FineffScheduled(Fineff),
    RetaliationTriggered(AttackRequest),
    // ...
}
```

This lets combat be *mostly* pure — it returns a list of effects to apply, and the caller applies them. The recursive nature (followup attacks) is handled by processing the `followup_attacks` list iteratively.

**Trade-off**: This is a major redesign from the C++ approach. ~50% of melee-attack.cc would need restructuring. But it makes the system testable, replayable, and ECS-compatible.

### Alternative: Direct Port (Pragmatic)

Keep combat as a monolithic system that reads/writes world state directly, similar to C++. Use Bevy `SystemParam` for access:

```rust
fn resolve_melee_attack(
    attacker: ActorRef,
    defender: ActorRef,
    player: &mut ResMut<Player>,
    monsters: &mut Query<(&mut Health, &mut MonsterEnchantments, &Speed)>,
    env: &Res<Environment>,
    messages: &mut EventWriter<GameMessage>,
    // ...many more params
) {
    // Direct port of melee_attack::attack()
}
```

This is faster to port but harder to test and parallelize. Given the sequential nature of DCSS turns, this may be acceptable for Phase 1.
