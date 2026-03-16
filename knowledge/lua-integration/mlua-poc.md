# mlua Proof-of-Concept Results

## Summary: All Tests Pass

mlua 0.10 with `lua51` + `vendored` features works perfectly for DCSS Lua integration.

## Findings

### Two Independent States
`Lua::new()` creates fully independent states. Globals set in one don't appear in the other. This models DCSS's clua/dlua separation.

### UserData
`MapDef` as `LuaUserData` works cleanly:
- `add_fields` exposes struct fields readable from Lua (`m.name`)
- `add_methods` exposes callable methods (`m:tags_string()`)
- `borrow_mut::<MapDef>()` gives mutable access from Rust function closures
- Type safety enforced — wrong userdata type produces a Lua error

### Function Registration
`lua.create_function(|_, (map, args): (LuaAnyUserData, String)| { ... })` maps directly to the DCSS `LUAFN` pattern. The `(LuaAnyUserData, String)` tuple destructuring replaces manual `lua_gettop`/`luaL_checkstring` calls.

### init.lua Loads Successfully
`dat/dlua/init.lua` loaded without errors. It defines `echoall()` and `crawl.message()` which reference `util.foreach` and `crawl.mpr` — these don't exist yet but Lua's lazy binding means they don't error at load time, only at call time.

### API Mapping Pattern

| C++ (l-dgn.cc) | Rust (mlua) |
|----------------|-------------|
| `MAP(ls, 1, map)` | `(map, ...): (LuaAnyUserData, ...)` in closure args |
| `luaL_checkstring(ls, 2)` | Second element of arg tuple: `String` |
| `PLUARET(string, val)` | `Ok(val)` return |
| `luaL_Reg[]` array | `table.set("name", lua.create_function(...))` |
| `lua_newstate()` | `Lua::new()` |
| `clua.callfn("ready", 0, 0)` | `lua.globals().get::<Function>("ready")?.call(())` |

### Dependencies
```toml
mlua = { version = "0.10", features = ["lua51", "vendored"] }
```
`vendored` bundles Lua 5.1 source — no system Lua needed.
