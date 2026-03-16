use bevy::prelude::*;
use crate::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    Wall,
    Floor,
    ClosedDoor,
    OpenDoor,
    StairsDown,
}

impl Feature {
    pub fn is_passable(self) -> bool {
        matches!(self, Feature::Floor | Feature::OpenDoor | Feature::StairsDown)
    }
}

#[derive(Resource)]
pub struct TerrainGrid {
    pub cells: [[Feature; MAP_WIDTH]; MAP_HEIGHT],
}

impl TerrainGrid {
    pub fn get(&self, pos: Coord) -> Option<Feature> {
        if pos.in_bounds() {
            Some(self.cells[pos.y as usize][pos.x as usize])
        } else {
            None
        }
    }

    pub fn set(&mut self, pos: Coord, feature: Feature) {
        if pos.in_bounds() {
            self.cells[pos.y as usize][pos.x as usize] = feature;
        }
    }

    pub fn is_passable(&self, pos: Coord) -> bool {
        self.get(pos).is_some_and(|f| f.is_passable())
    }
}

/// Grid mapping coordinates to terrain sprite entities (for swapping tile images).
#[derive(Resource)]
pub struct TerrainSpriteGrid {
    pub cells: [[Option<Entity>; MAP_WIDTH]; MAP_HEIGHT],
}

impl Default for TerrainSpriteGrid {
    fn default() -> Self {
        Self {
            cells: [[None; MAP_WIDTH]; MAP_HEIGHT],
        }
    }
}

impl TerrainSpriteGrid {
    pub fn get(&self, pos: Coord) -> Option<Entity> {
        if pos.in_bounds() {
            self.cells[pos.y as usize][pos.x as usize]
        } else {
            None
        }
    }

    pub fn set(&mut self, pos: Coord, entity: Option<Entity>) {
        if pos.in_bounds() {
            self.cells[pos.y as usize][pos.x as usize] = entity;
        }
    }
}

/// Convert .des map glyph to a terrain feature.
pub fn glyph_to_feature(ch: char) -> Feature {
    match ch {
        'x' | 'X' | 'c' | 'v' | 'b' => Feature::Wall,
        '.' | '{' | '}' | '(' | ')' | '[' | ']' | '@' => Feature::Floor,
        '+' => Feature::ClosedDoor,
        '>' => Feature::StairsDown,
        ' ' => Feature::Wall, // space = rock/wall
        // Water, lava, etc. — treat as floor for now
        'w' | 'W' | 'l' => Feature::Floor,
        // Trees, statues — treat as wall
        't' | 'G' => Feature::Wall,
        // Monster/item glyphs — floor underneath
        '0'..='9' | 'A'..='Z' | 'a'..='z' => Feature::Floor,
        // Default: anything else is floor
        _ => Feature::Floor,
    }
}

/// Create a TerrainGrid from .des map lines.
/// The map is centered in the grid. Returns the grid and the player start position.
pub fn from_map_lines(lines: &[String]) -> (TerrainGrid, Coord) {
    let mut cells = [[Feature::Wall; MAP_WIDTH]; MAP_HEIGHT];
    let map_height = lines.len().min(MAP_HEIGHT);
    let map_width = lines.iter().map(|l| l.len()).max().unwrap_or(0).min(MAP_WIDTH);

    // Center the vault in the grid
    let offset_y = (MAP_HEIGHT.saturating_sub(map_height)) / 2;
    let offset_x = (MAP_WIDTH.saturating_sub(map_width)) / 2;

    let mut player_pos = Coord::new(offset_x as i32 + 1, offset_y as i32 + 1);

    for (y, line) in lines.iter().enumerate() {
        if y >= MAP_HEIGHT {
            break;
        }
        for (x, ch) in line.chars().enumerate() {
            if x >= MAP_WIDTH {
                break;
            }
            let grid_x = offset_x + x;
            let grid_y = offset_y + y;
            if grid_x < MAP_WIDTH && grid_y < MAP_HEIGHT {
                cells[grid_y][grid_x] = glyph_to_feature(ch);
                if ch == '{' || ch == '@' {
                    player_pos = Coord::new(grid_x as i32, grid_y as i32);
                }
            }
        }
    }

    (TerrainGrid { cells }, player_pos)
}

/// Create a hardcoded multi-room dungeon for the MVP.
pub fn hardcoded_dungeon() -> TerrainGrid {
    use Feature::*;
    let mut cells = [[Wall; MAP_WIDTH]; MAP_HEIGHT];

    // Room 1: top-left (2,2) to (12,8)
    for y in 2..=8 {
        for x in 2..=12 {
            cells[y][x] = Floor;
        }
    }

    // Room 2: top-right (20,2) to (35,8)
    for y in 2..=8 {
        for x in 20..=35 {
            cells[y][x] = Floor;
        }
    }

    // Room 3: bottom-left (2,12) to (15,18)
    for y in 12..=18 {
        for x in 2..=15 {
            cells[y][x] = Floor;
        }
    }

    // Room 4: bottom-right (22,12) to (37,17)
    for y in 12..=17 {
        for x in 22..=37 {
            cells[y][x] = Floor;
        }
    }

    // Corridor: Room 1 → Room 2 (horizontal at y=5)
    for x in 13..=19 {
        cells[5][x] = Floor;
    }

    // Corridor: Room 1 → Room 3 (vertical at x=7)
    for y in 9..=11 {
        cells[y][7] = Floor;
    }

    // Corridor: Room 2 → Room 4 (vertical at x=28)
    for y in 9..=11 {
        cells[y][28] = Floor;
    }

    // Corridor: Room 3 → Room 4 (horizontal at y=15)
    for x in 16..=21 {
        cells[15][x] = Floor;
    }

    // Doors
    cells[5][13] = ClosedDoor;
    cells[9][7] = ClosedDoor;
    cells[15][16] = OpenDoor;

    // Stairs down in room 4
    cells[14][30] = StairsDown;

    TerrainGrid { cells }
}
