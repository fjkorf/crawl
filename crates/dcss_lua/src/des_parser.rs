//! Hand-written .des file parser.
//!
//! Parses .des files into Vec<RawMapDef>, where each RawMapDef contains:
//! - Accumulated Lua source (from keywords converted to Lua calls + embedded Lua blocks)
//! - Raw MAP lines
//!
//! The Lua source is later executed against the dgn.* API to populate a MapDef.

/// A raw parsed vault definition from a .des file.
/// Contains Lua source chunks and raw map lines, not yet executed.
#[derive(Debug, Clone, Default)]
pub struct RawMapDef {
    /// Lua source for the "main" chunk — accumulated from keywords and `: ` lines.
    pub main_lua: String,
    /// Lua source for PRELUDE {{ }} block.
    pub prelude_lua: String,
    /// Lua source for VALIDATE {{ }} block.
    pub validate_lua: String,
    /// Lua source for VETO {{ }} block.
    pub veto_lua: String,
    /// Lua source for EPILOGUE {{ }} block.
    pub epilogue_lua: String,
    /// Raw ASCII map lines between MAP and ENDMAP.
    pub map_lines: Vec<String>,
    /// Source file and line number of NAME: directive.
    pub source_file: String,
    pub source_line: usize,
}

/// Global prelude Lua source from {{ }} blocks outside any NAME: definition.
#[derive(Debug, Clone, Default)]
pub struct DesFileResult {
    pub global_prelude: String,
    pub maps: Vec<RawMapDef>,
    pub default_depth: String,
}

/// Parser state machine.
#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    /// Outside any vault definition.
    TopLevel,
    /// Inside a vault definition (after NAME:, before next NAME: or EOF).
    InVault,
    /// Inside MAP..ENDMAP block.
    InMap,
    /// Inside {{ }} Lua block.
    InLuaBlock(LuaBlockTarget),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LuaBlockTarget {
    GlobalPrelude,
    Main,
    Prelude,
    Validate,
    Veto,
    Epilogue,
}

/// Parse a .des file into raw map definitions.
pub fn parse_des_file(content: &str, filename: &str) -> Result<DesFileResult, String> {
    let mut result = DesFileResult::default();
    let mut state = State::TopLevel;
    let mut current_map = RawMapDef::default();
    let mut has_vault = false;
    let mut continuation = String::new();

    for (line_num, raw_line) in content.lines().enumerate() {
        let line_num = line_num + 1; // 1-indexed

        // Handle line continuation
        if raw_line.ends_with('\\') {
            continuation.push_str(&raw_line[..raw_line.len() - 1]);
            continuation.push(' ');
            continue;
        }
        let line = if continuation.is_empty() {
            raw_line.to_string()
        } else {
            let mut full = std::mem::take(&mut continuation);
            full.push_str(raw_line);
            full
        };

        // Strip comments (but not inside Lua blocks or map blocks)
        let line = match state {
            State::InLuaBlock(_) | State::InMap => line,
            _ => strip_comment(&line),
        };

        let trimmed = line.trim();
        if trimmed.is_empty() && state != State::InMap && !matches!(state, State::InLuaBlock(_)) {
            continue;
        }

        match state {
            State::TopLevel => {
                if trimmed.starts_with("{{") {
                    state = State::InLuaBlock(LuaBlockTarget::GlobalPrelude);
                    // If there's code after {{ on the same line
                    let rest = trimmed[2..].trim();
                    if !rest.is_empty() && rest != "}}" {
                        result.global_prelude.push_str(rest);
                        result.global_prelude.push('\n');
                    }
                    if rest == "}}" {
                        state = State::TopLevel;
                    }
                } else if let Some(value) = strip_keyword(trimmed, "default-depth") {
                    result.default_depth = value.to_string();
                } else if let Some(value) = strip_keyword(trimmed, "NAME") {
                    // Start a new vault
                    current_map = RawMapDef {
                        source_file: filename.to_string(),
                        source_line: line_num,
                        ..Default::default()
                    };
                    current_map
                        .main_lua
                        .push_str(&format!("name(\"{}\")\n", escape_lua_string(value)));
                    has_vault = true;
                    state = State::InVault;
                }
            }
            State::InVault => {
                if let Some(value) = strip_keyword(trimmed, "NAME") {
                    // Finish previous vault, start new one
                    result.maps.push(std::mem::take(&mut current_map));
                    current_map = RawMapDef {
                        source_file: filename.to_string(),
                        source_line: line_num,
                        ..Default::default()
                    };
                    current_map
                        .main_lua
                        .push_str(&format!("name(\"{}\")\n", escape_lua_string(value)));
                } else if trimmed == "MAP" {
                    state = State::InMap;
                } else if trimmed.starts_with("{{") {
                    state = State::InLuaBlock(LuaBlockTarget::Main);
                    let rest = trimmed[2..].trim();
                    if !rest.is_empty() && rest != "}}" {
                        current_map.main_lua.push_str(rest);
                        current_map.main_lua.push('\n');
                    }
                    if rest == "}}" {
                        state = State::InVault;
                    }
                } else if trimmed.starts_with(':') {
                    // Lua one-liner
                    let lua_code = trimmed[1..].trim();
                    current_map.main_lua.push_str(lua_code);
                    current_map.main_lua.push('\n');
                } else if let Some((target, rest)) = parse_lua_block_keyword(trimmed) {
                    state = State::InLuaBlock(target);
                    if !rest.is_empty() && rest != "}}" {
                        get_lua_target(&mut current_map, target).push_str(rest);
                        get_lua_target(&mut current_map, target).push('\n');
                    }
                    if rest == "}}" {
                        state = State::InVault;
                    }
                } else {
                    // Try to parse as a keyword directive
                    parse_vault_keyword(trimmed, &mut current_map);
                }
            }
            State::InMap => {
                if trimmed == "ENDMAP" {
                    state = State::InVault;
                } else {
                    current_map.map_lines.push(raw_line.to_string());
                }
            }
            State::InLuaBlock(target) => {
                if trimmed.starts_with("}}") {
                    state = if has_vault {
                        State::InVault
                    } else {
                        State::TopLevel
                    };
                } else {
                    let dest = match target {
                        LuaBlockTarget::GlobalPrelude => &mut result.global_prelude,
                        _ => get_lua_target(&mut current_map, target),
                    };
                    dest.push_str(&line);
                    dest.push('\n');
                }
            }
        }
    }

    // Finish last vault if any
    if has_vault && (!current_map.main_lua.is_empty() || !current_map.map_lines.is_empty()) {
        result.maps.push(current_map);
    }

    Ok(result)
}

fn get_lua_target(map: &mut RawMapDef, target: LuaBlockTarget) -> &mut String {
    match target {
        LuaBlockTarget::GlobalPrelude => unreachable!(),
        LuaBlockTarget::Main => &mut map.main_lua,
        LuaBlockTarget::Prelude => &mut map.prelude_lua,
        LuaBlockTarget::Validate => &mut map.validate_lua,
        LuaBlockTarget::Veto => &mut map.veto_lua,
        LuaBlockTarget::Epilogue => &mut map.epilogue_lua,
    }
}

fn parse_lua_block_keyword(line: &str) -> Option<(LuaBlockTarget, &str)> {
    for (keyword, target) in [
        ("prelude", LuaBlockTarget::Prelude),
        ("validate", LuaBlockTarget::Validate),
        ("veto", LuaBlockTarget::Veto),
        ("epilogue", LuaBlockTarget::Epilogue),
    ] {
        if let Some(rest) = line.strip_prefix(keyword) {
            let rest = rest.trim();
            if rest.starts_with("{{") {
                return Some((target, rest[2..].trim()));
            }
        }
    }
    None
}

/// Parse a vault keyword line and generate the corresponding Lua call.
fn parse_vault_keyword(line: &str, map: &mut RawMapDef) {
    let keywords: &[(&str, &str)] = &[
        ("TAGS", "tags"),
        ("DEPTH", "depth"),
        ("PLACE", "place"),
        ("ORIENT", "orient"),
        ("WEIGHT", "weight"),
        ("CHANCE", "chance"),
        ("DESC", "desc"),
        ("ORDER", "order"),
        ("MONS", "mons"),
        ("ITEM", "item"),
        ("KFEAT", "kfeat"),
        ("KITEM", "kitem"),
        ("KMONS", "kmons"),
        ("KPROP", "kprop"),
        ("KMASK", "kmask"),
        ("SUBST", "subst"),
        ("NSUBST", "nsubst"),
        ("SHUFFLE", "shuffle"),
        ("COLOUR", "colour"),
        ("FTILE", "ftile"),
        ("RTILE", "rtile"),
        ("TILE", "tile"),
        ("CLEAR", "clear"),
        ("FHEIGHT", "fheight"),
        ("SUBVAULT", "subvault"),
        ("MARKER", "marker"),
    ];

    for &(keyword, lua_fn) in keywords {
        if let Some(value) = strip_keyword(line, keyword) {
            map.main_lua.push_str(&format!(
                "{}(\"{}\")\n",
                lua_fn,
                escape_lua_string(value)
            ));
            return;
        }
    }
}

/// Strip a keyword prefix like "TAGS:" and return the value portion.
fn strip_keyword<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    let trimmed = line.trim();
    if trimmed.len() > keyword.len()
        && trimmed[..keyword.len()].eq_ignore_ascii_case(keyword)
        && trimmed.as_bytes().get(keyword.len()) == Some(&b':')
    {
        Some(trimmed[keyword.len() + 1..].trim())
    } else {
        None
    }
}

/// Strip # comments from a line (outside Lua/map blocks).
fn strip_comment(line: &str) -> String {
    // Don't strip # inside quoted strings (simplified: just find first #)
    if let Some(pos) = line.find('#') {
        line[..pos].to_string()
    } else {
        line.to_string()
    }
}

/// Escape a string for embedding in Lua source.
fn escape_lua_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn parse_simple_vault() {
        let des = r#"
NAME:   test_vault
TAGS:   arrival transparent
ORIENT: float
MAP
xxxxx
x...x
x.{.x
x...x
xxxxx
ENDMAP
"#;
        let result = parse_des_file(des, "test.des").unwrap();
        assert_eq!(result.maps.len(), 1);
        let map = &result.maps[0];
        assert!(map.main_lua.contains("name(\"test_vault\")"));
        assert!(map.main_lua.contains("tags(\"arrival transparent\")"));
        assert!(map.main_lua.contains("orient(\"float\")"));
        assert_eq!(map.map_lines.len(), 5);
        assert_eq!(map.map_lines[2], "x.{.x");
    }

    #[test]
    fn parse_multiple_vaults() {
        let des = r#"
NAME:   vault_one
TAGS:   one
MAP
...
ENDMAP

NAME:   vault_two
TAGS:   two
MAP
xxx
ENDMAP
"#;
        let result = parse_des_file(des, "test.des").unwrap();
        assert_eq!(result.maps.len(), 2);
        assert!(result.maps[0].main_lua.contains("name(\"vault_one\")"));
        assert!(result.maps[1].main_lua.contains("name(\"vault_two\")"));
    }

    #[test]
    fn parse_lua_oneliner() {
        let des = r#"
NAME:   lua_test
: if true then
TAGS:   conditional
: end
MAP
...
ENDMAP
"#;
        let result = parse_des_file(des, "test.des").unwrap();
        assert_eq!(result.maps.len(), 1);
        let lua = &result.maps[0].main_lua;
        assert!(lua.contains("if true then"));
        assert!(lua.contains("tags(\"conditional\")"));
        assert!(lua.contains("end"));
    }

    #[test]
    fn parse_lua_block() {
        let des = r#"
{{
function helper()
  return true
end
}}

NAME:   block_test
MAP
...
ENDMAP
"#;
        let result = parse_des_file(des, "test.des").unwrap();
        assert!(result.global_prelude.contains("function helper()"));
        assert_eq!(result.maps.len(), 1);
    }

    #[test]
    fn parse_validate_veto_epilogue() {
        let des = r#"
NAME:   chunks_test
MAP
...
ENDMAP
validate {{
  return has_exit_from_glyph('O')
}}
veto {{
  return false
}}
epilogue {{
  crawl.mpr("placed!")
}}
"#;
        let result = parse_des_file(des, "test.des").unwrap();
        let map = &result.maps[0];
        assert!(map.validate_lua.contains("has_exit_from_glyph"));
        assert!(map.veto_lua.contains("return false"));
        assert!(map.epilogue_lua.contains("crawl.mpr"));
    }

    #[test]
    fn parse_real_des_file() {
        // Try from workspace root (cargo test) or crate dir
        let path = ["crawl-ref/source/dat/des/arrival/simple.des",
                     "../../crawl-ref/source/dat/des/arrival/simple.des"]
            .iter()
            .find(|p| std::path::Path::new(p).exists())
            .expect("cannot find simple.des from either workspace root or crate dir");
        let content = std::fs::read_to_string(path)
            .expect("failed to read simple.des");
        let result = parse_des_file(&content, "simple.des").unwrap();
        assert!(
            result.maps.len() > 5,
            "expected many vaults, got {}",
            result.maps.len()
        );
        // First vault should be minmay_arrival_doors
        assert!(result.maps[0].main_lua.contains("minmay_arrival_doors"));
        // All vaults should have map lines
        for (i, map) in result.maps.iter().enumerate() {
            assert!(
                !map.map_lines.is_empty(),
                "vault {} has no map lines",
                i
            );
        }
        println!(
            "Parsed {} vaults from simple.des",
            result.maps.len()
        );
    }

    #[test]
    fn parse_lair_des() {
        let path = ["crawl-ref/source/dat/des/branches/lair.des",
                     "../../crawl-ref/source/dat/des/branches/lair.des"]
            .iter()
            .find(|p| std::path::Path::new(p).exists())
            .expect("cannot find lair.des");
        let content = std::fs::read_to_string(path).expect("failed to read lair.des");
        let result = parse_des_file(&content, "lair.des").unwrap();
        assert!(
            result.maps.len() > 10,
            "expected many vaults in lair.des, got {}",
            result.maps.len()
        );
        // lair.des has a global Lua prelude with helper functions
        println!(
            "Parsed {} vaults from lair.des, global prelude: {} bytes",
            result.maps.len(),
            result.global_prelude.len()
        );
    }

    #[test]
    fn parse_all_des_files() {
        let des_dir = ["crawl-ref/source/dat/des", "../../crawl-ref/source/dat/des"]
            .iter()
            .find(|p| std::path::Path::new(p).is_dir())
            .expect("cannot find dat/des directory");

        let mut total_vaults = 0;
        let mut total_files = 0;
        let mut errors = Vec::new();

        for entry in walkdir(des_dir) {
            if !entry.ends_with(".des") {
                continue;
            }
            total_files += 1;
            let content = match std::fs::read_to_string(&entry) {
                Ok(c) => c,
                Err(e) => {
                    errors.push(format!("{}: read error: {}", entry, e));
                    continue;
                }
            };
            match parse_des_file(&content, &entry) {
                Ok(result) => total_vaults += result.maps.len(),
                Err(e) => errors.push(format!("{}: parse error: {}", entry, e)),
            }
        }

        println!("Parsed {} vaults from {} .des files", total_vaults, total_files);
        if !errors.is_empty() {
            println!("Errors ({}):", errors.len());
            for e in &errors {
                println!("  {}", e);
            }
        }
        assert!(errors.is_empty(), "{} parse errors", errors.len());
        assert!(total_vaults > 500, "expected 500+ vaults, got {}", total_vaults);
    }
}

/// Recursively list all files under a directory.
fn walkdir(dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walkdir(path.to_str().unwrap_or("")));
            } else if let Some(s) = path.to_str() {
                files.push(s.to_string());
            }
        }
    }
    files
}
