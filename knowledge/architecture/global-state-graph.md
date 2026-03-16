# DCSS Global State Graph

## Overview

DCSS has four primary global state objects, plus ~20 secondary globals. All game logic reads/writes these freely — there is no encapsulation boundary between subsystems.

## Primary Globals

### `env` — `crawl_environment` (env.h)
The entire dungeon level state. Contains:

**Entity Storage (fixed arrays)**:
- `mons[MAX_MONSTERS+2]` — All monsters on current level. Indexed by `mindex()`. Last 2 slots are "anon" (temporary).
- `item[MAX_ITEMS]` — All items on current level. Floor items form linked lists via `item.link`.
- `max_mon_index` — Optimization: highest occupied monster slot.

**Spatial Grids (GXM x GYM fixed arrays)**:
- `grid` — Terrain features (walls, floors, doors, etc.)
- `pgrid` — Terrain properties (bitfield: haloed, bloody, etc.)
- `mgrid` — Monster-to-coordinate mapping (unsigned short → mindex)
- `igrid` — Item-to-coordinate mapping (int → item index, head of linked list)
- `grid_colours` — Colour overrides per cell

**Map Knowledge (player's view)**:
- `map_knowledge` — What the player has seen/remembers (MapKnowledge = FixedArray<map_cell, GXM, GYM>)
- `map_forgotten` — Forgotten map data (Ctrl+F)
- `map_seen` — Bitmask of explored cells
- `visible` — Set of currently visible coordinates

**Level Metadata**:
- `cloud` — Map of coord → cloud_struct (fog, fire, etc.)
- `shop` — Map of coord → shop_struct
- `trap` — Map of coord → trap_def
- `markers` — Map markers (triggers, portal entries, etc.)
- `level_vaults` — Vector of vault_placement (placed vaults with positions/sizes)
- `level_map_mask` / `level_map_ids` — Vault placement masks
- `level_uniq_maps`, `level_uniq_map_tags` — Uniqueness tracking for map generation
- `properties` — CrawlHashTable for arbitrary per-level data

**Level Timing**:
- `elapsed_time`, `turns_on_level`, `dactions_done`
- `sanctuary_pos`, `sanctuary_time`, `orb_pos`
- `forest_awoken_until`, `forest_is_hostile`
- `density`, `absdepth0`

**Volatile (not serialized)**:
- `level_state` — Flags for current level state
- `mid_cache` — mid→mindex mapping during transitions
- `final_effect_monster_cache` — Cached monsters for blame after death
- `new_subvault_names/tags` — Vault placement stack during generation

### `you` — `player` (player.h)
The player character. Inherits from `actor`. ~200+ fields organized as:

**Identity**: `your_name`, `species`, `char_class`, `wizard`, `explore`, `birth_time`

**Core Stats**:
- `hp`, `hp_max`, `hp_max_adj_temp`, `hp_max_adj_perm`
- `magic_points`, `max_magic_points`, `mp_max_adj`
- `base_stats[NUM_STATS]` (str/int/dex)
- `experience`, `experience_level`, `max_level`, `gold`

**Character Build**:
- `skills[NUM_SKILLS]`, `train[NUM_SKILLS]`, `skill_points[NUM_SKILLS]`, `training[NUM_SKILLS]`
- `spells[MAX_KNOWN_SPELLS]`, `spell_library[NUM_SPELLS]`
- `mutation[NUM_MUTATIONS]`, `innate_mutation[NUM_MUTATIONS]`, `temp_mutation[NUM_MUTATIONS]`
- `attribute[NUM_ATTRIBUTES]`, `duration[NUM_DURATIONS]`

**Equipment & Inventory**:
- `inv[ENDOFPACK+1]` — Player inventory (FixedVector of item_def)
- `equipment` — player_equip_set mapping slots → inv indices
- `runes[NUM_RUNE_TYPES]`, `gems_found/shattered[NUM_GEM_TYPES]`

**Religion**:
- `religion` (god_type), `raw_piety`, `piety_hysteresis`, `gift_timeout`
- `penance[NUM_GODS]`, `worshipped[NUM_GODS]`, `num_current_gifts[NUM_GODS]`
- `one_time_ability_used[NUM_GODS]`, `piety_max[NUM_GODS]`

**Form/Transformation**: `form`, `default_form`, `cur_talisman`, `symbol`

**Exploration**: `where_are_you` (branch_type), `depth`, `uniq_map_tags/names`, `vault_list`

**UI/Volatile State**: `turn_is_over`, `banished`, `redraw_*` flags, `flash_colour`, `time_taken`, `delay_queue`

**Timing**: `elapsed_time`, `real_time_ms`, `num_turns`, `exploration`

### `crawl_state` — `game_state` (state.h)
Meta-game state and command processing:

**Game Phase**: `game_started`, `need_save`, `saving_game`, `generating_level`, `type` (normal/sprint/arena/tutorial/descent)

**Command Processing**: `waiting_for_command`, `prev_cmd`, `repeat_cmd`, `cmd_repeat_start`, `doing_prev_cmd_again`

**Active Actor Tracking**: `mon_act` (currently acting monster), `god_act` (currently acting god) — both with stacks

**UI State**: `mouse_enabled`, `io_inited`, `show_more_prompt`, `darken_range`, `flash_monsters`

**Debug/Meta**: `game_crashed`, `seed`, `map_stat_gen`, `obj_stat_gen`, `test`, `script`

### `Options` — `game_options` (options.h)
User configuration. ~1000+ lines of preferences. Not gameplay state — read-only during play.

## Secondary Globals

| Global | Type | Location | Purpose |
|--------|------|----------|---------|
| `clua` | CLua | clua.h | User script Lua interpreter |
| `dlua_errors` | vector<CLuaError> | clua.h | Dungeon Lua error log |
| `branches[NUM_BRANCHES]` | const Branch[] | branch.h | Branch definitions (static data) |
| `brentry[NUM_BRANCHES]` | FixedVector<level_id> | branch.h | Branch entry points |
| `brdepth[NUM_BRANCHES]` | FixedVector<int> | branch.h | Branch depths |
| `branch_bribe[NUM_BRANCHES]` | FixedVector<int> | branch.h | Bribe amounts |
| `the_lost_ones` | monsters_in_transit | mon-transit.h | Monsters between levels |
| `travel_cache` | TravelCache | travel.h | Travel system cache |
| `shopping_list` | ShoppingList | shopping.h | Player shopping list |
| `dungeon_events` | dgn_event_dispatcher | dgn-event.h | Dungeon event system |
| `inv_order` | FixedVector<int> | invent.h | Inventory display order |
| `active_monster_band` | band_type | mon-place.h | Current band being placed |
| `Temp_Vaults` | vector<vault_placement> | dungeon.h | Temp vault storage |
| `lc_loaded_maps` etc. | various | maps.h | Level compiler state |

## Key Architectural Observations

### 1. Index-Based Cross-References
Everything references everything by index into fixed arrays:
- Monsters reference items by index into `env.item[]`
- `env.mgrid[x][y]` maps coordinates to monster indices in `env.mons[]`
- `env.igrid[x][y]` maps coordinates to item indices in `env.item[]`
- Items chain via `item.link` (index of next item in pile)
- Player inventory is `you.inv[]`, equipment maps to indices within that

### 2. CrawlHashTable (String-Keyed Properties)
Both `actor.props`, `item_def.props`, and `env.properties` use `CrawlHashTable` — a string-keyed hash map storing heterogeneous values. Used extensively for ad-hoc state that doesn't warrant a dedicated field (dozens of `#define *_KEY` constants).

### 3. Actor Polymorphism
`player` and `monster` both inherit from `actor` (80+ virtual methods). Code frequently uses `actor*` to handle both uniformly. Key shared interface: position, movement, combat stats, resistances, status effects, visibility.

### 4. Subsystem Access Patterns
Every major subsystem reads multiple globals:
- **Combat** (attack.cc, fight.cc): reads attacker/defender (actor*), env.grid for terrain effects, you for player-specific bonuses
- **Monster AI** (mon-act.cc): reads env.mons for all monsters, env.grid/mgrid for pathfinding, you for player position/visibility
- **Dungeon gen** (dungeon.cc): writes env.grid/pgrid/mgrid/igrid, env.mons, env.item, reads you.where_are_you/depth
- **Rendering** (view.cc, tileview.cc): reads env.map_knowledge, env.grid, env.mons, you.pos, crawl_state
- **Spells** (spl-*.cc): reads/writes you (spell costs, effects), env (area effects, monster targeting)

### 5. No Subsystem Boundaries
There are no modules, namespaces, or access control between subsystems. Any function can read/write any global. This is the primary challenge for ECS decomposition — there's no natural component boundary.

## ECS Mapping Implications

### Natural Entities
- **Monsters** → Entity with components (Position, Health, MonsterType, AIState, Enchantments, Inventory, Speed)
- **Items** (floor) → Entity with components (Position, ItemType, Properties)
- **Clouds** → Entity with components (Position, CloudType, Duration)
- **Traps** → Entity with components (Position, TrapType)
- **Shops** → Entity with components (Position, ShopInventory)

### Natural Resources
- **Player** → Single Resource (too many fields and too frequently accessed to decompose)
- **Terrain Grid** → Resource (FixedArray<feature_type, GXM, GYM>) — doesn't change often
- **Game State** → Resource (crawl_state equivalent)
- **Options** → Resource (read-only)
- **Map Knowledge** → Resource (player's view of the world)

### The Index Problem
The biggest challenge: DCSS uses array indices as entity IDs everywhere. Converting to Bevy Entity IDs requires:
1. Replacing all `mindex()` lookups with Entity queries
2. Replacing `env.mgrid[x][y]` spatial lookups with spatial queries or a maintained grid Resource
3. Replacing item linked lists with Entity relationships
4. Maintaining a `HashMap<OldIndex, Entity>` during translation, or restructuring all cross-references

### Recommended Hybrid Approach
- Monsters and items as ECS Entities (natural fit for queries, spawn/despawn)
- Maintain a `MonsterGrid` Resource (`FixedArray<Option<Entity>, GXM, GYM>`) for spatial lookups
- Player as a singleton Resource (too interconnected to decompose)
- Terrain as a Resource (rarely changes, frequently read)
- Use Bevy `Events` for turn phase transitions
