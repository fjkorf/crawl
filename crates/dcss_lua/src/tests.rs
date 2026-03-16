//! Tests for Lua integration: state management, parser, and execution pipeline.

use crate::des_parser;
use crate::lua_state::*;
use mlua::prelude::*;

#[test]
fn create_independent_lua_states() {
    let dlua = create_dlua().expect("failed to create dlua");
    let clua = create_clua().expect("failed to create clua");

    dlua.globals().set("test_var", 42).unwrap();
    let dlua_val: i32 = dlua.globals().get("test_var").unwrap();
    assert_eq!(dlua_val, 42);

    let clua_val: LuaValue = clua.globals().get("test_var").unwrap();
    assert!(clua_val.is_nil());
}

#[test]
fn execute_raw_map_simple() {
    let lua = create_dlua().unwrap();

    let raw = des_parser::RawMapDef {
        main_lua: r#"
            name("test_vault")
            tags("arrival transparent")
            depth("D:1-5")
            orient("float")
            weight("30")
            kfeat("O = enter_lair")
            mons("goblin, orc, gnoll")
        "#
        .to_string(),
        map_lines: vec![
            "xxxxx".into(),
            "x...x".into(),
            "x.{.x".into(),
            "x...x".into(),
            "xxxxx".into(),
        ],
        ..Default::default()
    };

    let map = execute_raw_map(&lua, &raw).unwrap();
    assert_eq!(map.name, "test_vault");
    assert_eq!(map.tags, vec!["arrival", "transparent"]);
    assert_eq!(map.depth, "D:1-5");
    assert_eq!(map.orient, "float");
    assert_eq!(map.weight, 30);
    assert_eq!(map.kfeat.get(&'O'), Some(&"enter_lair".to_string()));
    assert_eq!(map.mons, vec!["goblin", "orc", "gnoll"]);
    assert_eq!(map.map_lines.len(), 5);
}

#[test]
fn parse_and_execute_des_file() {
    let des = r#"
NAME:   test_doors
TAGS:   arrival no_monster_gen transparent
ORIENT: float
MAP
     x+x
     x.x
     x+x
xxxxx...xxxxx
+.+.+.{.+.+.+
xxxxx...xxxxx
     x+x
     x.x
     x+x
ENDMAP

NAME:   test_simple
TAGS:   arrival transparent
ORIENT: float
MONS:   plant
MAP
xxx
x{x
xxx
ENDMAP
"#;

    let parsed = des_parser::parse_des_file(des, "test.des").unwrap();
    assert_eq!(parsed.maps.len(), 2);

    let lua = create_dlua().unwrap();

    let map1 = execute_raw_map(&lua, &parsed.maps[0]).unwrap();
    assert_eq!(map1.name, "test_doors");
    assert!(map1.tags.contains(&"arrival".to_string()));
    assert_eq!(map1.orient, "float");
    assert_eq!(map1.map_lines.len(), 9);

    let map2 = execute_raw_map(&lua, &parsed.maps[1]).unwrap();
    assert_eq!(map2.name, "test_simple");
    assert_eq!(map2.mons, vec!["plant"]);
    assert_eq!(map2.map_lines.len(), 3);
}

#[test]
fn parse_and_execute_real_simple_des() {
    let path = find_des_path("dat/des/arrival/simple.des");
    let content = std::fs::read_to_string(&path).expect("failed to read");
    let parsed = des_parser::parse_des_file(&content, "simple.des").unwrap();

    let lua = create_dlua().unwrap();

    let mut success = 0;
    let mut failures = Vec::new();

    for raw in &parsed.maps {
        match execute_raw_map(&lua, raw) {
            Ok(map) => {
                assert!(!map.name.is_empty(), "vault has empty name");
                success += 1;
            }
            Err(e) => {
                failures.push(format!("{}:{}: {}", raw.source_file, raw.source_line, e));
            }
        }
    }

    println!(
        "simple.des: {}/{} vaults executed ({} failures)",
        success,
        parsed.maps.len(),
        failures.len()
    );
    for f in &failures[..failures.len().min(10)] {
        println!("  FAIL: {}", f);
    }

    // Most vaults should succeed — some may fail due to missing Lua functions
    let success_rate = success as f64 / parsed.maps.len() as f64;
    assert!(
        success_rate > 0.5,
        "expected >50% success rate, got {:.0}% ({}/{})",
        success_rate * 100.0,
        success,
        parsed.maps.len()
    );
}

#[test]
fn userdata_field_access() {
    let lua = create_dlua().unwrap();
    let map = lua.create_userdata(MapDef {
        name: "test".into(),
        tags: vec!["a".into(), "b".into()],
        ..Default::default()
    }).unwrap();
    lua.globals().set("m", map).unwrap();

    let name: String = lua.load("return m.name").eval().unwrap();
    assert_eq!(name, "test");

    let tags: String = lua.load("return m:tags_string()").eval().unwrap();
    assert_eq!(tags, "a b");
}

#[test]
fn lua_conditional_execution() {
    let lua = create_dlua().unwrap();

    let raw = des_parser::RawMapDef {
        main_lua: r#"
            name("cond_vault")
            local depth = 3
            if depth < 5 then
                tags("shallow")
            else
                tags("deep")
            end
        "#
        .to_string(),
        map_lines: vec!["...".into()],
        ..Default::default()
    };

    let map = execute_raw_map(&lua, &raw).unwrap();
    assert_eq!(map.name, "cond_vault");
    assert_eq!(map.tags, vec!["shallow"]);
}

#[test]
fn execute_all_des_files() {
    let des_dir = find_des_path("dat/des");
    let dlua_dir = find_des_path("dat/dlua");
    let lua = create_dlua_with_base(&dlua_dir).unwrap();

    // Load Lua helper libraries
    let _ = load_lua_file(&lua, &format!("{}/init.lua", dlua_dir));
    let _ = load_lua_file(&lua, &format!("{}/vault.lua", dlua_dir));
    if let Ok(entries) = std::fs::read_dir(&dlua_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("lm_") && name.ends_with(".lua") {
                let _ = load_lua_file(&lua, &format!("{}/{}", dlua_dir, name));
            }
        }
    }

    // Pass 1: parse all .des files and execute global preludes
    let mut all_parsed = Vec::new();
    for entry in des_parser::walkdir(&des_dir) {
        if !entry.ends_with(".des") {
            continue;
        }
        let content = match std::fs::read_to_string(&entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let parsed = match des_parser::parse_des_file(&content, &entry) {
            Ok(r) => r,
            Err(_) => continue,
        };
        // Execute global prelude (defines helper functions like lair_entry_setup)
        if !parsed.global_prelude.is_empty() {
            let _ = lua.load(&parsed.global_prelude).set_name(&entry).exec();
        }
        all_parsed.push(parsed);
    }

    // Pass 2: execute all vault definitions
    let mut total_vaults = 0;
    let mut total_success = 0;
    let mut error_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for parsed in &all_parsed {
        for raw in &parsed.maps {
            total_vaults += 1;
            match execute_raw_map(&lua, raw) {
                Ok(_) => total_success += 1,
                Err(e) => {
                    // Extract the missing function name from the error
                    let msg = e.to_string();
                    let key = if let Some(start) = msg.find("call global '") {
                        let rest = &msg[start + 13..];
                        if let Some(end) = rest.find('\'') {
                            format!("missing: {}", &rest[..end])
                        } else {
                            "other".into()
                        }
                    } else {
                        "other".into()
                    };
                    *error_counts.entry(key).or_insert(0) += 1;
                }
            }
        }
    }

    let pct = total_success as f64 / total_vaults as f64 * 100.0;
    println!(
        "Executed {}/{} vaults ({:.1}%) across all .des files",
        total_success, total_vaults, pct
    );

    if !error_counts.is_empty() {
        let mut sorted: Vec<_> = error_counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        println!("Failure breakdown:");
        for (reason, count) in &sorted[..sorted.len().min(20)] {
            println!("  {:4} — {}", count, reason);
        }
    }

    // 75%+ with current stubs; will improve as more Lua APIs are implemented
    assert!(pct > 70.0, "expected >70% success, got {:.1}%", pct);
}

fn find_des_path(relative: &str) -> String {
    let candidates = [
        format!("crawl-ref/source/{}", relative),
        format!("../../crawl-ref/source/{}", relative),
    ];
    candidates
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .cloned()
        .unwrap_or_else(|| panic!("cannot find {}", relative))
}
