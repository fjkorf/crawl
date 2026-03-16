# .des Parser Port — Findings

## Parser Design

Hand-written line-oriented Rust parser — no parser generator needed. The .des grammar is simple enough for a state machine with 4 states: TopLevel, InVault, InMap, InLuaBlock.

### Key Design Decisions

1. **Keywords → Lua function calls**: Matches the C++ Bison approach. `TAGS: foo bar` becomes `tags("foo bar")` in a Lua chunk string. This means the parser output is Lua source, not structured data.

2. **Bare global functions, not `dgn.method`**: The C++ parser generates calls to bare globals like `name()`, `tags()`, `depth()`, not `dgn.name()`. The Lua state registers these as globals that operate on an implicit "current map" via the Lua registry.

3. **Two-pass loading**: Pass 1 parses all files and executes global Lua preludes (defining helper functions). Pass 2 executes individual vault Lua chunks. This is required because helpers like `lair_entry_setup()` are defined in file A's prelude but called from file B's vaults.

## Results

- **Parser**: 5,996 vaults parsed from 143 .des files, zero parse errors
- **Execution**: 4,520/5,996 (75.4%) execute with current API stubs
- **End-to-end**: Real vaults from simple.des render in Bevy with correct tile layout

## Glyph Mapping

```
x, X, c, v, b, space → Wall
., {, }, (, ), [, ], @ → Floor
+ → ClosedDoor
> → StairsDown
w, W, l → Floor (water/lava — visual only for now)
t, G → Wall (trees, statues)
0-9, A-Z, a-z → Floor (monster/item placement markers)
```

## What's Still Needed

- `SUBST` / `NSUBST` / `SHUFFLE` processing (glyph substitution)
- Monster placement from `MONS` / `KMONS` definitions
- `MARKER` Lua execution for fog machines, triggers, etc.
- `validate` / `veto` chunk execution for vault placement validation
- Remaining 25% of vaults need more API stubs (landmark constructors, etc.)
