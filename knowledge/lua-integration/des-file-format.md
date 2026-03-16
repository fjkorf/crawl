# .des File Format Specification

## Overview

.des files define dungeon vaults using a **hybrid format**: ~70% custom DSL keywords, ~30% embedded Lua code. A custom parser (`mapdef.cc`, 6.5K lines) handles the DSL portion.

## File Organization

```
dat/des/
├── 00init.des           # Initialization vaults
├── branches/            # Branch-specific (lair.des, elf.des, zot.des, etc.)
├── variable/            # Floating minivaults
├── altar/               # Altar vaults
├── arrival/             # D:1 arrival vaults
├── portals/             # Portal vault definitions
├── serial/              # Serial vault chains
├── traps/               # Trap vaults
├── builder/             # Layout construction
├── sprint/ & tutorial/  # Game mode specific
└── test.des             # Testing
```

**Scale**: 197K lines of .des files total.

## DSL Syntax (Parsed by mapdef.cc)

### Vault Header Keywords

```des
NAME:    vault_name           # Required. Unique identifier.
TAGS:    tag1 tag2            # Metadata: no_rotate, transparent, unrand, etc.
DEPTH:   D:5-10, Lair         # Where vault can appear (branch:depth ranges)
PLACE:   Lair:$               # Specific placement (branch:level, $ = last)
ORIENT:  float                # north/south/east/west/float/encompass/centre
WEIGHT:  30                   # Selection weight (higher = more common)
CHANCE:  5%                   # Selection chance
```

### Content Definition Keywords

```des
MONS:    orc / gnoll, ogre   # Monster slots 1-9 (comma-separated, / = random choice)
ITEM:    short sword, potion  # Item slots a-h
KMONS:   P = plant / fungus   # Keyed: glyph P becomes plant or fungus
KFEAT:   O = enter_lair       # Keyed: glyph O becomes lair entrance
KITEM:   d = gold             # Keyed: glyph d gets gold
KPROP:   . = no_tele_into     # Keyed: glyph . gets property
KMASK:   . = no_item          # Keyed: glyph . mask
```

### Map Transformation Keywords

```des
SUBST:   A = x:5 .            # Replace A with wall (5x weight) or floor
NSUBST:  1 = 2:12 / 2:1 / *=. # Numbered: first 2→"12", next 2→"1", rest→"."
SHUFFLE: 234, abcd             # Randomly permute glyphs within groups
COLOUR:  x = blue              # Glyph color override
FTILE:   - = floor_grass       # Floor tile assignment
RTILE:   x = wall_brick        # Rock tile assignment
TILE:    G = dngn_statue        # General tile assignment
```

### Map Block

```des
MAP
xxxxxxxxxxxxx
x...........x
x.1.........x
x...........x
x.....O.....x
x...........x
xxxxxxxxxxxxx
ENDMAP
```

Glyphs: `x`=wall, `.`=floor, `@`=player entry, `0-9`=monster slots, `A-Z`/`a-z`=keyed overrides

## Embedded Lua

### Top-level Lua Blocks (between vaults)
```des
{{
crawl_require('dlua/vault.lua')

function my_helper(e)
    e.tags("my_tag")
    e.kfeat("O = enter_lair")
end
}}
```

### Inline Lua (`: ` prefix, within vault definition)
```des
NAME: my_vault
: my_helper(_G)
: if you.absdepth() > 10 then
MONS: dragon
: else
MONS: wyvern
: end
MAP
...
ENDMAP
```

### Lua Markers (in MARKER keyword)
```des
MARKER: A = lua:fog_machine { cloud_type = "flame", pow_min = 5 }
MARKER: B = lua:one_way_stair { dst = "target_level" }
```

## Lua Libraries (dat/dlua/)

| File | Size | Purpose |
|------|------|---------|
| init.lua | small | Core initialization |
| loadmaps.lua | 12 lines | Loops through .des files, calls dgn.load_des_file() |
| vault.lua | ~150 lines | Common helper functions |
| lm_trig.lua | 24.7KB | Triggerable event system (largest) |
| lm_trove.lua | 21.2KB | Trove vault generation |
| lm_fog.lua | 16.8KB | Fog machine effects |
| lm_tmsg.lua | 6.3KB | Timed messages |
| lm_timed.lua | 4.2KB | Timed events |
| lm_monst.lua | 3.5KB | Monster spawning |
| lm_door.lua | 3.2KB | Door mechanics |
| lm_replica.lua | 2.7KB | Replica markers |
| lm_mon_prop.lua | 1.7KB | Monster properties |
| lm_1way.lua | 1.5KB | One-way stairs |
| lm_items.lua | 972B | Item placement |
| Others | small | Various helpers |

## Parsed Structure: map_def (mapdef.h)

Each vault becomes a `map_def` object:
```cpp
class map_def {
    string name;
    depth_ranges place, depths;
    map_section_type orient;
    range_chance_t _chance;
    range_weight_t _weight;

    map_lines map;              // The ASCII grid
    mons_list mons;             // MONS definitions
    item_list items;            // ITEM definitions

    // Lua code sections:
    dlua_chunk prelude;         // Runs before map processing
    dlua_chunk mapchunk;        // Runs during map processing
    dlua_chunk main;            // Main Lua block
    dlua_chunk validate;        // Validation code
    dlua_chunk veto;            // Veto code (can reject placement)
    dlua_chunk epilogue;        // Runs after placement

    unordered_set<string> tags;
    colour_t rock_colour, floor_colour;
    string rock_tile, floor_tile;
    map<dungeon_feature_type, string> feat_renames;
};
```

## Port Strategy

### Option A: Port the .des Parser to Rust (Recommended)
- Rewrite `mapdef.cc` keyword parsing in Rust (~6.5K lines)
- Use `mlua` for embedded Lua execution
- Keep .des files unchanged — full backwards compatibility
- Effort: ~2-3 weeks for parser, ~4-6 weeks for Lua API bindings

### Option B: Compile .des to Pure Lua
- Write a transpiler that converts DSL keywords to Lua function calls
- e.g., `MONS: orc` → `dgn.mons("orc")`
- Eliminates need for custom parser
- Risk: Some keywords have complex syntax (NSUBST, SUBST with weights)

### Option C: New Format (Not Recommended)
- Would require rewriting 197K lines of vault definitions
- Loses community contribution compatibility
- No clear benefit over Option A

### Key Insight
The .des format is well-designed and battle-tested. The parser is complex but finite (6.5K lines). The Lua API is large (230 functions) but each function is simple (typically <30 lines). The pragmatic path is to port the parser and API bindings, keeping all .des files intact.
