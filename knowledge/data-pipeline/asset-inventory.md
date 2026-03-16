# DCSS Data Pipeline — Asset Inventory

## Bevy-Ready (YAML, direct serde deserialization)

### Monster Definitions
- **Path**: `dat/mons/*.yaml`
- **Size**: 2.6 MB, 672 files, 12,619 lines
- **Schema**: name, glyph, flags, resists, exp, attacks, hd, hp_10x, ac, ev, spells, intelligence, speed, size, shape
- **Action**: Define Rust structs, `serde_yaml::from_str()` directly

### Species Definitions
- **Path**: `dat/species/*.yaml`
- **Size**: 196 KB, 49 files, 2,639 lines
- **Schema**: enum, monster, name, difficulty, aptitudes, str/int/dex, levelup_stat_frequency, recommended_jobs, mutations
- **Action**: Define Rust structs, direct deserialization

### Job Definitions
- **Path**: `dat/jobs/*.yaml`
- **Size**: 108 KB, 27 files, 727 lines
- **Schema**: enum, name, category, str/int/dex, equipment, weapon_choice, recommended_species, skills
- **Action**: Define Rust structs, direct deserialization

## Needs Custom Parser

### Tile Assets
- **Path**: `rltiles/`
- **Size**: 49 MB, 8,417 PNG files
- **Descriptors**: 21 `.txt` files (dc-mon.txt, dc-feat.txt, dc-item.txt, dc-wall.txt, dc-floor.txt, etc.)
- **Format**: `%sdir` for source directory, `%name` for page name, `filename ENUM_NAME` per tile
- **Current build**: C++ tool (`tile_list_processor.cc`, 63KB) packs PNGs into sprite pages
- **Action**: Write Rust build script or Bevy asset loader that:
  1. Parses `.txt` descriptors
  2. Loads PNGs and creates `TextureAtlas` assets
  3. Generates enum-to-atlas-index mapping

### Spell Data
- **Path**: `spl-data.h`
- **Size**: 87 KB
- **Format**: C++ static array of `spell_desc` structs
- **Fields**: enum, name, schools (bitfield), flags, level, power_cap, range, noise, tile
- **Action**: One-time extraction script (regex on C++ source) → YAML/RON file

### Feature Descriptions
- **Path**: `dat/descript/features.txt`
- **Size**: 43 KB
- **Format**: `%%%%` separated entries, plain text descriptions
- **Action**: Simple regex splitter → HashMap<String, String>

### Database Files
- **Path**: `dat/database/`
- **Size**: 1.2 MB total
- **Key files**: monspeak.txt (210KB), monspell.txt, randname.txt, godspeak.txt, randbook.txt
- **Format**: Custom DSL with `%%%%` separators, `@template@` substitutions, `w:N` weighting
- **Action**: Port the template expansion system (~500 lines). Critical for flavor text.

### Artifact Definitions
- **Path**: `art-data.txt`
- **Size**: 52 KB
- **Format**: Custom field-per-line DSL. Currently processed by `util/art-data.pl` (Perl)
- **Fields**: NAME, OBJ, COLOUR, PLUS, BRAND, properties, TILE, etc.
- **Action**: Rewrite Perl extractor in Rust or convert to YAML once

## Data Table Extraction (C++ Headers → YAML/RON)

These C++ static arrays need one-time extraction:

| File | Size | Content |
|------|------|---------|
| `spl-data.h` | 87 KB | Spell definitions (~514 entries) |
| `mutation-data.h` | ~50 KB | Mutation definitions (~269 entries) |
| `zap-data.h` | ~50 KB | Zap/beam definitions |
| `mon-spell.h` | ~50 KB | Monster spell lists |
| `feature-data.h` | ~15 KB | Feature properties |
| `duration-data.h` | ~20 KB | Duration descriptions |
| `mon-pick-data.h` | ~30 KB | Monster generation rates by depth |

**Strategy**: Write a single extraction tool that uses regex to parse C++ struct initializers and outputs YAML. Then load via `serde_yaml` at runtime or `include!` at compile time.

## Priority for Vertical Slice

1. **Monster YAML** — needed for any monster rendering
2. **Tile assets** — needed for dungeon display
3. **Feature data** — needed for terrain rendering
4. **Spell data** — needed for combat prototype
5. **Species/Job YAML** — needed for character creation
6. Everything else can wait for later phases
