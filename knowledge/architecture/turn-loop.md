# DCSS Turn Loop Architecture

## Overview

DCSS uses a synchronous, sequential turn loop. Each turn consists of:
1. Player input and action execution
2. World reaction (all monsters act, environment effects, time advancement)

The main loop lives in `main.cc:_input()` (line 1080), called infinitely from `_launch_game()` (line 498).

## Single Turn Flow

```
_input() [main.cc:1080]
│
├── Housekeeping: clear monster flags, religion updates, revival check, apply exp
│
├── If player cannot act (paralyzed/asleep/vexed):
│   ├── Handle vexed attack if applicable
│   ├── time_taken = stun_delay
│   └── world_reacts() → RETURN
│
├── Autopickup (may consume turn → world_reacts → RETURN)
│
├── If delay queue active (multi-turn actions):
│   ├── handle_delay() — executes current delay (casting, stairs, etc.)
│   ├── If turn consumed: world_reacts() → RETURN
│   └── Else: RETURN (instantaneous delay step)
│
├── COMMAND LOOP:
│   ├── Call Lua ready() hook
│   ├── _get_next_cmd() — BLOCKS for player input
│   ├── process_command(cmd) — executes player action
│   │   Sets you.time_taken and you.turn_is_over
│   └── Loop until non-repeat command
│
├── If you.turn_is_over:
│   ├── Berserk penalty check
│   ├── Handle channelled spells
│   └── world_reacts() ← THE BIG ONE
│
└── Else (instantaneous action): viewwindow() + update_screen()
```

## world_reacts() [main.cc:2599]

Called after every time-consuming player action:

```
world_reacts()
│
├── fire_final_effects() — scheduled explosions, etc.
├── update_monsters_in_view()
├── descent_crumble_stairs() — if time passed
├── you.turn_is_over = true, religion_turn_end()
├── check_banished()
├── run_environment_effects() — clouds, poison, lava
├── player_reacts() — trap triggers, status checks
├── abyss_morph()
├── apply_noises() — monster reactions to noise
│
├── handle_monsters(true) ← MONSTER TURNS
│   ├── Build priority queue sorted by speed_increment
│   ├── While queue not empty:
│   │   ├── Pop highest priority monster
│   │   ├── handle_monster_move(mon)
│   │   ├── fire_final_effects()
│   │   └── Re-queue if still has energy
│   └── apply_noises()
│
├── fire_final_effects() — post-monster effects
├── Elapsed time tracking: you.elapsed_time += you.time_taken
├── handle_time() — decrement durations, hunger
├── manage_clouds() — decay/spread
├── player_reacts_to_monsters()
├── clear_monster_flags()
├── viewwindow() + update_screen() — RENDER
└── Turn counter++, periodic save
```

## Monster Turn Processing [mon-act.cc]

### Energy/Speed System
- Each monster has `speed` (base) and `speed_increment` (energy counter)
- Normal speed = 10, speed_increment starts ~100
- Increments each turn based on speed stat
- Action when `speed_increment >= ENERGY_THRESHOLD`
- Actions cost energy: movement ~10, attack ~15 (varies)
- Faster monsters get multiple actions per player turn

### handle_monster_move() [mon-act.cc:1928]
Massive function with many special cases:
1. Validate monster (alive, has energy)
2. Handle special monster types (projectiles, prisms, tentacles, boulders, etc.)
3. Apply passive damage (clouds, walls)
4. Regeneration
5. Check incapacitation (paralyzed, dazed, vexed)
6. Standard AI: pathfinding → spell selection → melee/ranged → movement

### Monster Priority Queue
- `MonsterActionQueueCompare` sorts by `speed_increment` descending
- Allows faster monsters to act first
- Monsters can re-queue for multiple actions if they have remaining energy
- This models speed differences: a hasted monster gets ~2 actions per player turn

## Key State Variables

| Variable | Type | Meaning |
|----------|------|---------|
| `you.turn_is_over` | bool | Player action consumed time, world should react |
| `you.time_taken` | int | Action duration in 1/10 turn units (10 = 1 full turn) |
| `you.elapsed_time` | int | Total game time |
| `you.num_turns` | int | Turn counter |
| `env.turns_on_level` | int | Turns spent on current level |
| `monster.speed` | int | Base speed (10 = normal) |
| `monster.speed_increment` | int | Energy counter |

## Delay System (Multi-Turn Actions)
- `you.delay_queue` — vector of pending Delay objects
- Types: MemoriseDelay, AscendingStairsDelay, DescendingStairsDelay, TransformDelay, RunDelay, RestDelay, etc.
- Each delay has a duration and per-tick action
- Stairs: creates a 1-tick delay, then triggers level transition
- `handle_delay()` processes the front delay, may consume turn

## Level Transitions
1. Player presses '>' → `_take_stairs(true)` validates
2. Creates `DescendingStairsDelay` in delay queue
3. Next `_input()` processes delay → triggers level load
4. Current level state saved, new level generated or loaded
5. `world_reacts()` runs on new level

## Bevy ECS Schedule Mapping

### Recommended: Phase-Based System Sets

```
Schedule::GameTurn {
    Phase::PreTurn        — housekeeping, status updates, revival
    Phase::PlayerInput    — wait for input (Bevy event)
    Phase::PlayerAction   — execute command, set time_taken
    Phase::WorldReaction  — only if turn_is_over {
        Phase::FinalEffects
        Phase::EnvironmentEffects
        Phase::PlayerReacts
        Phase::MonsterTurns  — iterative: process monster queue
        Phase::PostMonster
        Phase::TimeAdvance
        Phase::CloudManage
        Phase::ViewUpdate
    }
}
```

### Key Constraint
The monster turn loop is iterative with re-queuing — a single monster may act multiple times. This cannot be naturally parallelized. The entire `MonsterTurns` phase must be sequential.

### Event-Driven Alternative
- `PlayerActionEvent` triggers world reaction chain
- `MonsterActionEvent` processed sequentially from priority queue
- `TurnCompleteEvent` triggers view update and save check
