# DCSS Enum Catalog for Rust Port

## Summary

15 core enums with ~2,700+ total variants need Rust equivalents.

## Enum Table

| Enum | File | Variants | Notes |
|------|------|----------|-------|
| `monster_type` | monster-type.h | ~1,075 | Largest enum. Genus markers, draconian sub-ranges, heavy versioning |
| `spell_type` | spell-type.h | ~514 | Range markers (FIRST_RANDOM..LAST_RANDOM), player-only vs monster-only |
| `command_type` | command-type.h | ~306 | Conditional: USE_TILE, __ANDROID__, TARGET_OS_MACOSX |
| `dungeon_feature_type` | dungeon-feature-type.h | ~271 | Must fit in u8 (COMPILE_CHECK <= 256). Altars, doors, stairs, walls |
| `mutation_type` | mutation-type.h | ~269 | Categories: RANDOM_GOOD, RANDOM_BAD, RANDOM_XOM, etc. |
| `duration_type` | duration-type.h | ~226 | Many paired: effect + cooldown |
| `enchant_type` | enchant-type.h | ~187 | Monster enchantments. Order matters (berserk implies haste+might) |
| `beam_type` | beam-type.h | ~112 | Range markers for random/enchantment/real categories |
| `attribute_type` | attribute-type.h | ~75 | Player attributes. God-specific entries |
| `species_type` | (auto-generated) | ~50 | Auto-gen from dat/species/. Draconian sub-range |
| `skill_type` | skill-type.h | ~48 | "NEVER rearrange" — save compat. Display-only variants |
| `branch_type` | branch-type.h | ~44 | Hell branch range markers |
| `job_type` | (auto-generated) | ~33 | Auto-gen from dat/jobs/ |
| `god_type` | god-type.h | ~31 | Small but critical. GOD_RANDOM=100, GOD_NAMELESS sentinels |
| `object_class_type` | object-class-type.h | ~23 | uint8_t. Item categories |

## Cross-Cutting Concerns

### Save Compatibility / Ordering
Many enums have explicit ordering requirements for save file compatibility:
- `skill_type`: "NEVER rearrange" comment
- `monster_type`: Hardcoded values, removed entries kept as placeholders
- `species_type`, `job_type`: Hardcoded values in generated code

**Rust approach**: Use `#[repr(u16)]` / `#[repr(u8)]` with explicit discriminants matching C++ values. This is critical only if we want save compatibility (likely not for the port).

### Conditional Compilation (TAG_MAJOR_VERSION)
C++ uses `#if TAG_MAJOR_VERSION == 34` extensively to maintain compatibility with old saves. Removed entries are kept in the enum but guarded.

**Rust approach**: For a clean port, drop all version-34 compatibility entries. Use only current-version variants. This simplifies every enum significantly (~10-20% fewer variants).

### Range Markers
Pattern: `FIRST_X = ..., ..., LAST_X = ...` for iteration over sub-ranges.

**Rust approach**: Use const arrays or `EnumIter` derive macro. For sub-ranges, define const slices or use `strum` crate's `EnumIter` with filter.

### Sentinel Values
Pattern: `FOO_RANDOM = 100`, `FOO_UNKNOWN = 1000` as special values outside the normal range.

**Rust approach**: Use `Option<EnumType>` for "none/unknown", separate enum variants for "random/viable", or a wrapper enum:
```rust
enum MonsterChoice {
    Specific(MonsterType),
    Random,
    RandomDraconian,
    RandomDemon(DemonTier),
}
```

### Bitfields
`species_flags` uses `DEF_BITFIELD` macro for bitfield flags.

**Rust approach**: Use `bitflags!` crate:
```rust
bitflags! {
    struct SpeciesFlags: u32 {
        const DRACONIAN = 1 << 0;
        const NO_HAIR = 1 << 1;
        // ...
    }
}
```

### Auto-Generated Types
Species and jobs are generated from Python scripts reading YAML data files.

**Rust approach**: Either:
1. `build.rs` script that reads the same YAML and generates Rust enums
2. Proc macro that reads YAML at compile time
3. Manual conversion (simpler, one-time effort for ~80 entries)

## Translation Priority

1. **Must have for vertical slice**: `monster_type`, `dungeon_feature_type`, `spell_type`, `beam_type`, `god_type`, `species_type`, `job_type`, `skill_type`, `command_type`
2. **Needed for full combat**: `enchant_type`, `duration_type`, `mutation_type`, `attribute_type`
3. **Nice to have**: `object_class_type` (items), `branch_type`

## Estimated Effort
- Mechanical translation of all 15 enums: ~2-3 days
- With supporting structs and data tables: ~1-2 weeks
- With auto-generation from YAML: additional ~1 week
