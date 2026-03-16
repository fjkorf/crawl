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

    pub fn is_passable(&self, pos: Coord) -> bool {
        self.get(pos).is_some_and(|f| f.is_passable())
    }
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
