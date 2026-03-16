//! Lua state management for DCSS.
//!
//! DCSS uses two independent Lua interpreters:
//! - `dlua` — dungeon Lua, for .des file execution and vault generation
//! - `clua` — client Lua, for user scripts and macros
//!
//! The .des parser converts keywords to Lua function calls like `name("vault_name")`,
//! `tags("arrival transparent")`, etc. These are bare globals (not `dgn.name`) that
//! operate on an implicit "current map" stored in the Lua registry.

use mlua::prelude::*;
use std::collections::HashMap;

use crate::des_parser::RawMapDef;
use crate::subst;

/// A parsed and Lua-executed map definition.
#[derive(Debug, Clone, Default)]
pub struct MapDef {
    pub name: String,
    pub tags: Vec<String>,
    pub depth: String,
    pub place: String,
    pub orient: String,
    pub weight: i32,
    pub chance: String,
    pub desc: String,
    pub order: i32,
    pub map_lines: Vec<String>,
    pub mons: Vec<String>,
    pub items: Vec<String>,
    pub kfeat: HashMap<char, String>,
    pub kmons: HashMap<char, String>,
    pub kitem: HashMap<char, String>,
    pub subst: Vec<String>,
    pub nsubst: Vec<String>,
    pub shuffle: Vec<String>,
    pub colour: Vec<String>,
    pub marker: Vec<String>,
}

impl LuaUserData for MapDef {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, this| Ok(this.name.clone()));
        fields.add_field_method_get("orient", |_, this| Ok(this.orient.clone()));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("tags_string", |_, this, ()| Ok(this.tags.join(" ")));
    }
}

/// Create the dungeon Lua state with vault-building functions registered.
pub fn create_dlua() -> LuaResult<Lua> {
    create_dlua_with_base("crawl-ref/source/dat/dlua")
}

/// Create dlua with a specific base path for Lua libraries.
pub fn create_dlua_with_base(dlua_base: &str) -> LuaResult<Lua> {
    let lua = Lua::new();
    register_vault_globals(&lua)?;
    register_dgn_table(&lua)?;
    register_crawl_table(&lua)?;
    register_you_table(&lua)?;
    register_util_table(&lua)?;
    register_crawl_require(&lua, dlua_base)?;
    Ok(lua)
}

/// Create the client Lua state (minimal for now).
pub fn create_clua() -> LuaResult<Lua> {
    Ok(Lua::new())
}

/// Execute a RawMapDef's Lua chunks against the dlua state, producing a MapDef.
pub fn execute_raw_map(lua: &Lua, raw: &RawMapDef) -> LuaResult<MapDef> {
    // Create a fresh MapDef and set it as the current map
    let map = MapDef {
        map_lines: raw.map_lines.clone(),
        ..Default::default()
    };
    let ud = lua.create_userdata(map)?;
    lua.set_named_registry_value("current_map", &ud)?;

    // Execute main Lua chunk (contains all keyword-generated calls)
    if !raw.main_lua.is_empty() {
        lua.load(&raw.main_lua)
            .set_name(&format!("{}:{}", raw.source_file, raw.source_line))
            .exec()
            .map_err(|e| {
                mlua::Error::external(format!(
                    "Error in vault at {}:{}: {}",
                    raw.source_file, raw.source_line, e
                ))
            })?;
    }

    // Extract the populated MapDef and apply glyph substitutions
    let mut result = ud.borrow::<MapDef>()?.clone();
    subst::apply_substitutions(
        &mut result.map_lines,
        &result.subst,
        &result.nsubst,
        &result.shuffle,
    );
    Ok(result)
}

/// Register bare global functions that the .des-generated Lua calls.
/// These operate on the "current_map" in the Lua registry.
fn register_vault_globals(lua: &Lua) -> LuaResult<()> {
    // Helper macro pattern: get current map, apply mutation
    macro_rules! reg_string_fn {
        ($lua:expr, $name:literal, $field:ident) => {
            $lua.globals().set(
                $name,
                $lua.create_function(|lua_ctx, value: String| {
                    let ud: LuaAnyUserData = lua_ctx.named_registry_value("current_map")?;
                    let mut map = ud.borrow_mut::<MapDef>()?;
                    map.$field = value;
                    Ok(())
                })?,
            )?;
        };
    }

    macro_rules! reg_vec_fn {
        ($lua:expr, $name:literal, $field:ident) => {
            $lua.globals().set(
                $name,
                $lua.create_function(|lua_ctx, value: String| {
                    let ud: LuaAnyUserData = lua_ctx.named_registry_value("current_map")?;
                    let mut map = ud.borrow_mut::<MapDef>()?;
                    map.$field.push(value);
                    Ok(())
                })?,
            )?;
        };
    }

    // Simple string fields
    reg_string_fn!(lua, "name", name);
    reg_string_fn!(lua, "depth", depth);
    reg_string_fn!(lua, "place", place);
    reg_string_fn!(lua, "orient", orient);
    reg_string_fn!(lua, "chance", chance);
    reg_string_fn!(lua, "desc", desc);

    // Tags: split on whitespace and add individually
    lua.globals().set(
        "tags",
        lua.create_function(|lua_ctx, value: String| {
            let ud: LuaAnyUserData = lua_ctx.named_registry_value("current_map")?;
            let mut map = ud.borrow_mut::<MapDef>()?;
            for tag in value.split_whitespace() {
                if !map.tags.contains(&tag.to_string()) {
                    map.tags.push(tag.to_string());
                }
            }
            Ok(())
        })?,
    )?;

    // Weight: parse as int
    lua.globals().set(
        "weight",
        lua.create_function(|lua_ctx, value: String| {
            let ud: LuaAnyUserData = lua_ctx.named_registry_value("current_map")?;
            let mut map = ud.borrow_mut::<MapDef>()?;
            map.weight = value.trim().parse().unwrap_or(10);
            Ok(())
        })?,
    )?;

    // Order: parse as int
    lua.globals().set(
        "order",
        lua.create_function(|lua_ctx, value: String| {
            let ud: LuaAnyUserData = lua_ctx.named_registry_value("current_map")?;
            let mut map = ud.borrow_mut::<MapDef>()?;
            map.order = value.trim().parse().unwrap_or(0);
            Ok(())
        })?,
    )?;

    // Mons: comma-separated list
    lua.globals().set(
        "mons",
        lua.create_function(|lua_ctx, value: String| {
            let ud: LuaAnyUserData = lua_ctx.named_registry_value("current_map")?;
            let mut map = ud.borrow_mut::<MapDef>()?;
            for m in value.split(',') {
                let trimmed = m.trim();
                if !trimmed.is_empty() {
                    map.mons.push(trimmed.to_string());
                }
            }
            Ok(())
        })?,
    )?;

    // Item: comma-separated list
    lua.globals().set(
        "item",
        lua.create_function(|lua_ctx, value: String| {
            let ud: LuaAnyUserData = lua_ctx.named_registry_value("current_map")?;
            let mut map = ud.borrow_mut::<MapDef>()?;
            for item in value.split(',') {
                let trimmed = item.trim();
                if !trimmed.is_empty() {
                    map.items.push(trimmed.to_string());
                }
            }
            Ok(())
        })?,
    )?;

    // Keyed features/monsters/items: "G = glyph_def"
    macro_rules! reg_keyed_fn {
        ($lua:expr, $name:literal, $field:ident) => {
            $lua.globals().set(
                $name,
                $lua.create_function(|lua_ctx, spec: String| {
                    let ud: LuaAnyUserData = lua_ctx.named_registry_value("current_map")?;
                    let mut map = ud.borrow_mut::<MapDef>()?;
                    if let Some((glyph_str, def)) = spec.split_once('=') {
                        if let Some(glyph) = glyph_str.trim().chars().next() {
                            map.$field.insert(glyph, def.trim().to_string());
                        }
                    }
                    Ok(())
                })?,
            )?;
        };
    }

    reg_keyed_fn!(lua, "kfeat", kfeat);
    reg_keyed_fn!(lua, "kmons", kmons);
    reg_keyed_fn!(lua, "kitem", kitem);

    // Accumulator fields (store raw spec strings for later processing)
    reg_vec_fn!(lua, "subst", subst);
    reg_vec_fn!(lua, "nsubst", nsubst);
    reg_vec_fn!(lua, "shuffle", shuffle);
    reg_vec_fn!(lua, "colour", colour);
    reg_vec_fn!(lua, "marker", marker);

    // Stubs for less-critical keywords (accept and ignore for now)
    for stub_name in [
        "ftile", "rtile", "tile", "clear", "fheight", "subvault", "kprop", "kmask",
        "set_feature_name", "lfloortile", "lrocktile",
    ] {
        lua.globals().set(
            stub_name,
            lua.create_function(|_, _value: String| Ok(()))?,
        )?;
    }

    Ok(())
}

/// Register the dgn.* table (used by Lua helper libraries like vault.lua).
fn register_dgn_table(lua: &Lua) -> LuaResult<()> {
    let dgn = lua.create_table()?;

    dgn.set("default_depth", lua.create_function(|_, _: String| Ok(()))?)?;

    // Stubs for functions called by vault.lua and .des files
    for stub in [
        "place_unique", "is_validating", "has_exit_from_glyph",
        "points_connected", "any_point_connected", "count_disconnected_zones",
    ] {
        dgn.set(stub, lua.create_function(|_, _: LuaMultiValue| Ok(false))?)?;
    }

    lua.globals().set("dgn", dgn)?;
    Ok(())
}

/// Register crawl.* table (game state queries used by Lua libraries).
fn register_crawl_table(lua: &Lua) -> LuaResult<()> {
    let crawl = lua.create_table()?;

    // Stubs returning safe defaults
    crawl.set("game_started", lua.create_function(|_, ()| Ok(false))?)?;
    crawl.set("random2", lua.create_function(|_, n: i32| Ok(rand_int(n)))?)?;
    crawl.set("one_chance_in", lua.create_function(|_, n: i32| Ok(rand_int(n) == 0))?)?;
    crawl.set("coinflip", lua.create_function(|_, ()| Ok(rand_int(2) == 0))?)?;
    crawl.set("x_chance_in_y", lua.create_function(|_, (x, y): (i32, i32)| Ok(rand_int(y) < x))?)?;
    crawl.set("random_range", lua.create_function(|_, (lo, hi): (i32, i32)| Ok(lo + rand_int(hi - lo + 1)))?)?;
    crawl.set("mpr", lua.create_function(|_, _msg: String| Ok(()))?)?;
    crawl.set("flush_prev_message", lua.create_function(|_, ()| Ok(()))?)?;
    crawl.set("delay", lua.create_function(|_, _n: i32| Ok(()))?)?;
    crawl.set("err_trace", lua.create_function(|_, _msg: String| Ok(()))?)?;
    crawl.set("stderr", lua.create_function(|_, _msg: String| Ok(()))?)?;

    lua.globals().set("crawl", crawl)?;
    Ok(())
}

/// Register you.* table (player state queries).
fn register_you_table(lua: &Lua) -> LuaResult<()> {
    let you = lua.create_table()?;

    you.set("depth", lua.create_function(|_, ()| Ok(1))?)?;
    you.set("absdepth", lua.create_function(|_, ()| Ok(1))?)?;
    you.set("branch", lua.create_function(|_, ()| Ok("D"))?)?;
    you.set("in_branch", lua.create_function(|_, _branch: String| Ok(false))?)?;
    you.set("god", lua.create_function(|_, ()| Ok("No God"))?)?;

    lua.globals().set("you", you)?;
    Ok(())
}

/// Register util.* table (utility functions used by Lua libraries).
fn register_util_table(lua: &Lua) -> LuaResult<()> {
    let util = lua.create_table()?;

    util.set("foreach", lua.create_function(|_, (tbl, func): (LuaTable, LuaFunction)| {
        for pair in tbl.pairs::<LuaValue, LuaValue>() {
            let (_, v) = pair?;
            func.call::<()>(v)?;
        }
        Ok(())
    })?)?;

    lua.globals().set("util", util)?;
    Ok(())
}

fn rand_int(n: i32) -> i32 {
    if n <= 0 { return 0; }
    // Simple RNG for Lua — not cryptographic, just for vault selection
    use std::time::SystemTime;
    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as i32;
    (seed.unsigned_abs() % n as u32) as i32
}

/// Register crawl_require() — loads Lua files from dat/dlua/.
fn register_crawl_require(lua: &Lua, base_path: &str) -> LuaResult<()> {
    let base = base_path.to_string();
    lua.globals().set(
        "crawl_require",
        lua.create_function(move |lua_ctx, path: String| {
            let full_path = format!("{}/{}", base, path);
            if std::path::Path::new(&full_path).exists() {
                let content = std::fs::read_to_string(&full_path)
                    .map_err(|e| mlua::Error::external(format!("{}: {}", full_path, e)))?;
                lua_ctx.load(&content).set_name(&full_path).exec()?;
            }
            Ok(())
        })?,
    )?;
    Ok(())
}

/// Load a Lua file into the given state.
pub fn load_lua_file(lua: &Lua, path: &str) -> LuaResult<()> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| mlua::Error::external(format!("Failed to read {}: {}", path, e)))?;
    lua.load(&content).set_name(path).exec()?;
    Ok(())
}
