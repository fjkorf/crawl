//! Unit tests for core game logic.

use crate::combat;
use crate::terrain::{self, Feature, TerrainGrid, from_map_lines, glyph_to_feature};
use crate::types::*;

#[test]
fn coord_adjacent() {
    let a = Coord::new(5, 5);
    assert!(a.adjacent_to(Coord::new(4, 4)));
    assert!(a.adjacent_to(Coord::new(6, 6)));
    assert!(a.adjacent_to(Coord::new(5, 6)));
    assert!(!a.adjacent_to(Coord::new(7, 5))); // dist 2
    assert!(!a.adjacent_to(Coord::new(5, 5))); // same pos
}

#[test]
fn coord_in_bounds() {
    assert!(Coord::new(0, 0).in_bounds());
    assert!(Coord::new(MAP_WIDTH as i32 - 1, MAP_HEIGHT as i32 - 1).in_bounds());
    assert!(!Coord::new(-1, 0).in_bounds());
    assert!(!Coord::new(MAP_WIDTH as i32, 0).in_bounds());
}

#[test]
fn feature_passability() {
    assert!(!Feature::Wall.is_passable());
    assert!(Feature::Floor.is_passable());
    assert!(!Feature::ClosedDoor.is_passable());
    assert!(Feature::OpenDoor.is_passable());
    assert!(Feature::StairsDown.is_passable());
}

#[test]
fn terrain_grid_get_set() {
    let mut grid = terrain::hardcoded_dungeon();
    let pos = Coord::new(5, 5);
    assert_eq!(grid.get(pos), Some(Feature::Floor));

    grid.set(pos, Feature::Wall);
    assert_eq!(grid.get(pos), Some(Feature::Wall));
    assert!(!grid.is_passable(pos));

    assert_eq!(grid.get(Coord::new(-1, -1)), None);
}

#[test]
fn glyph_mapping() {
    assert_eq!(glyph_to_feature('x'), Feature::Wall);
    assert_eq!(glyph_to_feature('.'), Feature::Floor);
    assert_eq!(glyph_to_feature('+'), Feature::ClosedDoor);
    assert_eq!(glyph_to_feature('>'), Feature::StairsDown);
    assert_eq!(glyph_to_feature('{'), Feature::Floor);
    assert_eq!(glyph_to_feature('@'), Feature::Floor);
    assert_eq!(glyph_to_feature(' '), Feature::Wall);
    assert_eq!(glyph_to_feature('1'), Feature::Floor); // monster slot
}

#[test]
fn from_map_lines_basic() {
    let lines = vec![
        "xxxxx".into(),
        "x...x".into(),
        "x.{.x".into(),
        "x...x".into(),
        "xxxxx".into(),
    ];
    let (grid, player_pos) = from_map_lines(&lines);

    // Player should be at the { position (offset into grid)
    assert_eq!(grid.get(player_pos), Some(Feature::Floor));

    // Check walls around the border of the vault
    let offset_x = (MAP_WIDTH - 5) / 2;
    let offset_y = (MAP_HEIGHT - 5) / 2;
    assert_eq!(grid.get(Coord::new(offset_x as i32, offset_y as i32)), Some(Feature::Wall));
    assert_eq!(grid.get(Coord::new(offset_x as i32 + 1, offset_y as i32 + 1)), Some(Feature::Floor));
}

#[test]
fn combat_always_produces_valid_results() {
    // Run many combats to verify no panics and reasonable ranges
    for _ in 0..1000 {
        let result = combat::resolve_melee(10, 15, 5, 10);
        assert!(result.damage >= 0);
        if result.hit {
            assert!(result.damage <= 10); // can't exceed base damage
        } else {
            assert_eq!(result.damage, 0);
        }
    }
}

#[test]
fn combat_zero_damage_weapon() {
    for _ in 0..100 {
        let result = combat::resolve_melee(0, 10, 0, 0);
        assert_eq!(result.damage, 0);
    }
}

#[test]
fn hardcoded_dungeon_has_rooms() {
    let grid = terrain::hardcoded_dungeon();
    // Room 1 center should be floor
    assert_eq!(grid.get(Coord::new(7, 5)), Some(Feature::Floor));
    // Room 2 center
    assert_eq!(grid.get(Coord::new(27, 5)), Some(Feature::Floor));
    // Corridor should be floor
    assert_eq!(grid.get(Coord::new(15, 5)), Some(Feature::Floor));
    // Door
    assert_eq!(grid.get(Coord::new(13, 5)), Some(Feature::ClosedDoor));
    // Stairs
    assert_eq!(grid.get(Coord::new(30, 14)), Some(Feature::StairsDown));
    // Outer wall
    assert_eq!(grid.get(Coord::new(0, 0)), Some(Feature::Wall));
}
