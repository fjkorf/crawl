# .des Parser Internals

## Architecture: Flex + Bison, NOT Hand-Written

The .des parser is a **Flex lexer + Bison LR(1) parser**, not a hand-written keyword dispatcher.

- Lexer: `crawl-ref/source/util/levcomp.lpp` (~310 lines)
- Parser: `crawl-ref/source/util/levcomp.ypp` (~687 lines)
- Prebuilt output: `crawl-ref/source/prebuilt/levcomp.tab.cc`, `levcomp.lex.cc`

### Lexer States

| State | Purpose |
|-------|---------|
| `MAPDEF` | Inside MAP..ENDMAP block — captures raw map lines |
| `LUA` | Multi-line `{{ }}` block — captures LUA_LINE tokens until `}}` |
| `LUA_ONELINER` | After `: ` prefix — captures one line of Lua |
| `ARGUMENT` | After NAME: etc. — captures rest of line as STRING |
| `KEYWORDS` | After TAGS: — captures space-separated keywords |
| `ITEM_LIST` | After ITEM: — handles comma-separated item specs |
| `MNAME` | After MONS: — handles comma-separated monster specs |

### Key Parsing Insight

**Metadata keywords generate Lua code, not data structures.** Example:

```
TAGS: minivault uniq
DEPTH: D:1-5
```

The Bison semantic actions convert these to Lua function calls:
```lua
tags("minivault uniq")
depth("D:1-5")
```

These are accumulated in the `map_def.main` dlua_chunk. When the map is later loaded for placement, the Lua chunks execute and populate the map_def struct via the dgn.* API.

### dlua_chunk

```cpp
class dlua_chunk {
    string chunk;      // Raw Lua source
    string compiled;   // Compiled bytecode (cached)
    int first, last;   // Source line numbers

    void add(int line, const string &text);  // Accumulate
    int load(CLua &interp);                   // Compile
    int run(CLua &interp);                    // Execute
};
```

Each map_def has 6 chunks: prelude, mapchunk, main, validate, veto, epilogue.

### Parsing Flow

```
dgn.load_des_file("file.des")
  → maps.cc: read_map() → _parse_maps()
    → fopen file
    → yyparse()  [Bison parser]
      → For each keyword: generate Lua code → accumulate in lc_map.main
      → For {{ }} blocks: accumulate in appropriate chunk (prelude/main/validate/veto/epilogue)
      → For MAP/ENDMAP: accumulate map lines in lc_map.mapchunk
      → On next NAME: or EOF: add_parsed_map(lc_map) → store in vdefs[]
    → Serialize to binary cache (.dsc, .idx, .lux)
```

### Binary Cache

Three files per .des source:
- `.lux` — Global prelude compiled bytecode
- `.dsc` — Full map data with serialized Lua chunks
- `.idx` — Index (map names, metadata, offsets for lazy loading)

## Porting Strategy Implications

The Flex/Bison approach means we have three options:

1. **Port the generated C to Rust** — Use the prebuilt `levcomp.tab.cc` and `levcomp.lex.cc` as reference. Rewrite the state machine in Rust. This is the most faithful approach.

2. **Use a Rust parser generator** — Write equivalent grammars for `lalrpop`, `pest`, or `nom`. The grammar is relatively simple (~30 keywords, ~10 lexer states).

3. **Hand-write a Rust parser** — The grammar is simple enough that a hand-written recursive descent parser would work. The lexer states map to an enum. This is probably the most maintainable.

4. **Keep using Flex/Bison via FFI** — Compile the existing C lexer/parser and call it from Rust. Fastest to get working but creates a C dependency.
