# Medium-Term Execution Plan

## Prioritized Order (dependencies flow top-down)

### 1. SUBST/NSUBST/SHUFFLE Processing (3 days)
Required for correct vault rendering. Many vaults look wrong without glyph substitution.
- Add to dcss_lua: process SUBST/NSUBST/SHUFFLE specs after Lua execution
- Simple string replacement on map_lines before converting to TerrainGrid
- Test: vault with SUBST renders correctly

### 2. Tile Atlas System (2 days)
Replace 15 individual PNGs with grouped loading. Not a full sprite sheet yet, but
organize tiles better and add more tile variants (water, lava, trees, statues).
- Add more TileId variants for terrain types revealed by vault rendering
- Load tiles in groups from rltiles/ subdirectories

### 3. Multi-Level Dungeon (1 week)
Level transitions via stairs. Core game loop enabler.
- LevelState resource storing TerrainGrid + monsters per level
- Stairs trigger level transition (generate new or restore saved)
- Random vault selection from simple.des for each new level
- Branch/depth tracking on Player

### 4. Character Creation (3 days)
Species/job selection using YAML data + litui screens.
- Load species YAML, job YAML
- litui define_markdown_app! for species/job selection screen
- Selected species/job sets starting stats, HP, MP

### 5. Enum Port — monster_type + dungeon_feature_type (3 days)
The two most critical enums for type safety.
- Extract from C++ headers to Rust enums with matching discriminants
- Replace String-based monster names with MonsterType enum
- Replace Feature enum with DungeonFeatureType (expanded set)

### 6. Items and Inventory (1 week)
Floor items, pickup, basic inventory screen.
- ItemDef struct matching DCSS item_def (simplified)
- Floor item entities with Position component
- Pickup on 'g' or auto-pickup
- egui inventory screen on 'i'
