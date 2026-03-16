//! Phase A: mlua proof-of-concept tests.

use crate::lua_state::*;
use mlua::prelude::*;

#[test]
fn create_independent_lua_states() {
    let dlua = create_dlua().expect("failed to create dlua");
    let clua = create_clua().expect("failed to create clua");

    // Set a global in dlua — should not appear in clua
    dlua.globals()
        .set("test_var", 42)
        .expect("failed to set global");

    let dlua_val: i32 = dlua.globals().get("test_var").expect("missing in dlua");
    assert_eq!(dlua_val, 42);

    let clua_val: LuaValue = clua.globals().get("test_var").expect("get failed");
    assert!(clua_val.is_nil(), "clua should not have dlua's global");
}

#[test]
fn register_and_call_dgn_stubs() {
    let lua = create_dlua().expect("failed to create dlua");

    // Create a MapDef userdata and set it as a global
    let map = lua
        .create_userdata(MapDef::default())
        .expect("failed to create userdata");
    lua.globals().set("test_map", map).expect("failed to set");

    // Call dgn functions from Lua
    lua.load(
        r#"
        dgn.name(test_map, "test_vault")
        dgn.tags(test_map, "minivault transparent")
        dgn.depth(test_map, "D:1-5")
        dgn.orient(test_map, "float")
        dgn.weight(test_map, 30)
        dgn.kfeat(test_map, "O = enter_lair")
        dgn.mons(test_map, "goblin, orc, gnoll")
    "#,
    )
    .exec()
    .expect("Lua execution failed");

    // Verify the MapDef was populated
    let map: LuaAnyUserData = lua.globals().get("test_map").unwrap();
    let map_ref = map.borrow::<MapDef>().unwrap();
    assert_eq!(map_ref.name, "test_vault");
    assert_eq!(map_ref.tags, vec!["minivault", "transparent"]);
    assert_eq!(map_ref.depth, "D:1-5");
    assert_eq!(map_ref.orient, "float");
    assert_eq!(map_ref.weight, 30);
    assert_eq!(map_ref.kfeat.get(&'O'), Some(&"enter_lair".to_string()));
    assert_eq!(map_ref.mons.len(), 3);
    assert_eq!(map_ref.mons[0], "goblin");
}

#[test]
fn userdata_methods_work() {
    let lua = create_dlua().expect("failed to create dlua");

    let map = lua
        .create_userdata(MapDef {
            name: "test".to_string(),
            tags: vec!["a".into(), "b".into()],
            ..Default::default()
        })
        .expect("failed to create userdata");
    lua.globals().set("m", map).expect("failed to set");

    // Access fields and methods from Lua
    let result: String = lua
        .load("return m.name")
        .eval()
        .expect("field access failed");
    assert_eq!(result, "test");

    let result: String = lua
        .load("return m:tags_string()")
        .eval()
        .expect("method call failed");
    assert_eq!(result, "a b");
}

#[test]
fn lua_execution_with_control_flow() {
    let lua = create_dlua().expect("failed to create dlua");

    let map = lua
        .create_userdata(MapDef::default())
        .expect("failed to create userdata");
    lua.globals().set("test_map", map).expect("failed to set");

    // Simulate conditional .des logic (: if/else pattern)
    lua.load(
        r#"
        local depth = 3
        if depth < 5 then
            dgn.tags(test_map, "shallow")
        else
            dgn.tags(test_map, "deep")
        end
        dgn.name(test_map, "conditional_vault")
    "#,
    )
    .exec()
    .expect("conditional Lua failed");

    let map: LuaAnyUserData = lua.globals().get("test_map").unwrap();
    let map_ref = map.borrow::<MapDef>().unwrap();
    assert_eq!(map_ref.name, "conditional_vault");
    assert_eq!(map_ref.tags, vec!["shallow"]);
}

#[test]
fn load_init_lua() {
    let lua = create_dlua().expect("failed to create dlua");

    // Try loading the real DCSS init.lua
    let result = load_lua_file(&lua, "crawl-ref/source/dat/dlua/init.lua");
    match result {
        Ok(()) => println!("init.lua loaded successfully"),
        Err(e) => {
            // init.lua may reference functions we haven't registered yet.
            // That's expected — document what's missing.
            println!("init.lua failed (expected at this stage): {}", e);
        }
    }
}
