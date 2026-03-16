# DCSS Lua API Surface Catalog

## Summary

- **28 binding files** (`l-*.cc`), 15,840 total lines
- **726 Lua-exposed functions** across all modules
- **Two interpreters**: CLua (user scripts), DLua (dungeon generation)
- **31 registration tables** (luaL_Reg arrays)

## Module Breakdown

### Critical for Dungeon Generation (must port first)

| Module | File | Functions | Purpose |
|--------|------|-----------|---------|
| dgn.* | l-dgn.cc | 106 | Main vault/map definition API |
| dgn.build.* | l-dgnbld.cc | 43 | Layout algorithms (maze, rooms, corridors) |
| dgn.grid.* | l-dgngrd.cc | 10 | Terrain/feature grid access |
| dgn.item.* | l-dgnit.cc | 9 | Item placement during generation |
| dgn.mons.* | l-dgnmon.cc | 7 | Monster placement during generation |
| dgn.level.* | l-dgnlvl.cc | 9 | Level-wide operations |
| dgn.event.* | l-dgnevt.cc | 9 | Dungeon event system |
| dgn.tile.* | l-dgntil.cc | varies | Tile assignment |
| dgn.subvault.* | l-subvault.cc | 4 | Subvault handling |
| mapmarker.* | l-mapmrk.cc | 4 | Map markers |
| mapgrd.* | l-mapgrd.cc | varies | Map grid metatables |
| feat.* | l-feat.cc | 22 | Feature/terrain properties |
| los.* | l-los.cc | 7 | Line-of-sight calculations |

**Total dungeon generation functions: ~230**

### Important for Gameplay Lua

| Module | File | Functions | Purpose |
|--------|------|-----------|---------|
| you.* | l-you.cc | 174 | Player properties & abilities |
| crawl.* | l-crawl.cc | 88 | Core game (I/O, RNG, input, display) |
| item.* | l-item.cc | 75 | Item manipulation |
| mons.* | l-mons.cc | 41 | Monster properties |
| moninf.* | l-moninf.cc | varies | Monster info display |
| spells.* | l-spells.cc | varies | Spell properties |
| view.* | l-view.cc | 15 | Screen view |
| travel.* | l-travel.cc | 10 | Travel/pathfinding |

### Less Critical

| Module | File | Functions | Purpose |
|--------|------|-----------|---------|
| debug.* | l-debug.cc | 30 | Debugging |
| option.* | l-option.cc | varies | Game options |
| file.* | l-file.cc | 12 | File I/O |
| colour.* | l-colour.cc | 1 | Color properties |
| autofight.* | l-autofight.cc | 3 | Autofight settings |
| wiz.* | l-wiz.cc | 3 | Wizard mode |
| global.* | l-global.cc | varies | Global utilities |

## Key dgn.* Functions (Most Critical)

### Map Definition (used in every .des file)
`name`, `tags`, `depth`, `place`, `orient`, `weight`, `chance`, `desc`, `order`
`mons`, `item`, `kfeat`, `kitem`, `kmons`, `kprop`, `kmask`
`subst`, `nsubst`, `shuffle`, `clear`, `colour`, `map`, `subvault`

### Map Analysis
`colour_at`, `fprop_at`, `cloud_at`, `terrain_changed`, `fprop_changed`
`points_connected`, `any_point_connected`, `has_exit_from`
`count_disconnected_zones`, `count_tele_zones`

### Vault Placement
`load_des_file`, `place_map`, `reuse_map`, `resolve_map`, `inspect_map`
`in_vault`, `set_map_mask`, `unset_map_mask`, `vault_at`
`find_marker_position_by_prop`, `find_marker_positions_by_prop`

### Builder Algorithms (l-dgnbld.cc)
**Shapes**: `make_circle`, `make_diamond`, `make_rounded_square`, `make_square`, `make_box`, `octa_room`
**Modification**: `replace_area`, `replace_first`, `replace_random`, `smear_map`, `spotty_map`, `add_pools`, `widen_paths`
**Algorithms**: `join_the_dots`, `delve`, `connect_adjacent_rooms`, `remove_isolated_glyphs`
**Layouts**: `layout_basic`, `layout_bigger_room`, `layout_chaotic_city`, `layout_shoals`, `layout_swamp`

## Binding Pattern

All functions use these macros from `cluautil.h`:
```cpp
LUAFN(name)        // → static int name(lua_State *ls)
LUAWRAP(name, expr) // Wraps with error handling
PLUARET(type, val)  // Push return value
GETCOORD(c, p1, p2, boundfn) // Extract coordinate from Lua args
```

Registration via `luaL_Reg` arrays:
```cpp
const struct luaL_Reg dgn_dlib[] = {
    { "name", dgn_name },
    { "tags", dgn_tags },
    // ...
    { nullptr, nullptr }
};
```

## Port Strategy for mlua

Each `luaL_Reg` entry maps directly to an `mlua` function registration:
```rust
// C++ original
LUAFN(dgn_name) {
    MAP(ls, 1, map);
    // ...
}

// Rust mlua equivalent
fn dgn_name(lua: &Lua, map: &MapDef) -> mlua::Result<String> {
    Ok(map.name.clone())
}

// Registration
let dgn = lua.create_table()?;
dgn.set("name", lua.create_function(dgn_name)?)?;
```

The 230 dungeon generation functions are the critical path. The 174 you.* functions and 88 crawl.* functions are needed for gameplay scripting but can be stubbed initially.
