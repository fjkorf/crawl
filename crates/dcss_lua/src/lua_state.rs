//! Lua state management for DCSS.
//!
//! DCSS uses two independent Lua interpreters:
//! - `dlua` — dungeon Lua, for .des file execution and vault generation
//! - `clua` — client Lua, for user scripts and macros
//!
//! Both are Lua 5.1 states managed via mlua.

use mlua::prelude::*;
use std::collections::HashMap;

/// A parsed map definition, populated by executing Lua chunks against the dgn.* API.
#[derive(Debug, Clone, Default)]
pub struct MapDef {
    pub name: String,
    pub tags: Vec<String>,
    pub depth: String,
    pub place: String,
    pub orient: String,
    pub weight: i32,
    pub chance: f64,
    pub map_lines: Vec<String>,
    pub mons: Vec<String>,
    pub items: Vec<String>,
    pub kfeat: HashMap<char, String>,
    pub kmons: HashMap<char, String>,
    pub kitem: HashMap<char, String>,
}

impl LuaUserData for MapDef {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, this| Ok(this.name.clone()));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("tags_string", |_, this, ()| {
            Ok(this.tags.join(" "))
        });
    }
}

/// Create the dungeon Lua state with the dgn.* API registered.
pub fn create_dlua() -> LuaResult<Lua> {
    let lua = Lua::new();
    register_dgn_api(&lua)?;
    Ok(lua)
}

/// Create the client Lua state (minimal for now).
pub fn create_clua() -> LuaResult<Lua> {
    let lua = Lua::new();
    // clua gets a more restricted API — stub for now
    Ok(lua)
}

/// Register the dgn.* table with core vault-building functions.
fn register_dgn_api(lua: &Lua) -> LuaResult<()> {
    let dgn = lua.create_table()?;

    // dgn.name(map, name_string)
    dgn.set(
        "name",
        lua.create_function(|_, (map, name): (LuaAnyUserData, String)| {
            let mut map = map.borrow_mut::<MapDef>()?;
            map.name = name;
            Ok(())
        })?,
    )?;

    // dgn.tags(map, tags_string)
    dgn.set(
        "tags",
        lua.create_function(|_, (map, tags): (LuaAnyUserData, String)| {
            let mut map = map.borrow_mut::<MapDef>()?;
            for tag in tags.split_whitespace() {
                if !map.tags.contains(&tag.to_string()) {
                    map.tags.push(tag.to_string());
                }
            }
            Ok(())
        })?,
    )?;

    // dgn.depth(map, depth_string)
    dgn.set(
        "depth",
        lua.create_function(|_, (map, depth): (LuaAnyUserData, String)| {
            let mut map = map.borrow_mut::<MapDef>()?;
            map.depth = depth;
            Ok(())
        })?,
    )?;

    // dgn.orient(map, orient_string)
    dgn.set(
        "orient",
        lua.create_function(|_, (map, orient): (LuaAnyUserData, String)| {
            let mut map = map.borrow_mut::<MapDef>()?;
            map.orient = orient;
            Ok(())
        })?,
    )?;

    // dgn.weight(map, weight_int)
    dgn.set(
        "weight",
        lua.create_function(|_, (map, weight): (LuaAnyUserData, i32)| {
            let mut map = map.borrow_mut::<MapDef>()?;
            map.weight = weight;
            Ok(())
        })?,
    )?;

    // dgn.kfeat(map, spec_string)  — e.g. "O = enter_lair"
    dgn.set(
        "kfeat",
        lua.create_function(|_, (map, spec): (LuaAnyUserData, String)| {
            let mut map = map.borrow_mut::<MapDef>()?;
            if let Some((glyph_str, feat)) = spec.split_once('=') {
                let glyph_str = glyph_str.trim();
                let feat = feat.trim().to_string();
                if let Some(glyph) = glyph_str.chars().next() {
                    map.kfeat.insert(glyph, feat);
                }
            }
            Ok(())
        })?,
    )?;

    // dgn.mons(map, mons_string)
    dgn.set(
        "mons",
        lua.create_function(|_, (map, mons): (LuaAnyUserData, String)| {
            let mut map = map.borrow_mut::<MapDef>()?;
            for m in mons.split(',') {
                map.mons.push(m.trim().to_string());
            }
            Ok(())
        })?,
    )?;

    lua.globals().set("dgn", dgn)?;
    Ok(())
}

/// Load a Lua file into the given state.
pub fn load_lua_file(lua: &Lua, path: &str) -> LuaResult<()> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| mlua::Error::external(format!("Failed to read {}: {}", path, e)))?;
    lua.load(&content).set_name(path).exec()?;
    Ok(())
}
