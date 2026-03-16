//! Field of View using recursive shadowcasting.
//! Calculates which tiles are visible from a given position.

use bevy::prelude::*;
use crate::terrain::{Feature, TerrainGrid};
use crate::types::*;

/// Tracks which cells are currently visible and which have been explored.
#[derive(Resource)]
pub struct VisibilityMap {
    pub visible: [[bool; MAP_WIDTH]; MAP_HEIGHT],
    pub explored: [[bool; MAP_WIDTH]; MAP_HEIGHT],
}

impl Default for VisibilityMap {
    fn default() -> Self {
        Self {
            visible: [[false; MAP_WIDTH]; MAP_HEIGHT],
            explored: [[false; MAP_WIDTH]; MAP_HEIGHT],
        }
    }
}

impl VisibilityMap {
    pub fn is_visible(&self, pos: Coord) -> bool {
        if pos.in_bounds() { self.visible[pos.y as usize][pos.x as usize] } else { false }
    }

    pub fn is_explored(&self, pos: Coord) -> bool {
        if pos.in_bounds() { self.explored[pos.y as usize][pos.x as usize] } else { false }
    }

    /// Recalculate FOV from the given position with the given radius.
    pub fn calculate(&mut self, origin: Coord, radius: i32, terrain: &TerrainGrid) {
        // Clear visible (keep explored)
        self.visible = [[false; MAP_WIDTH]; MAP_HEIGHT];

        // Origin is always visible
        self.set_visible(origin);

        // Cast shadows in 8 octants
        for octant in 0..8 {
            self.cast_light(terrain, origin, radius, 1, 1.0, 0.0, octant);
        }
    }

    fn set_visible(&mut self, pos: Coord) {
        if pos.in_bounds() {
            self.visible[pos.y as usize][pos.x as usize] = true;
            self.explored[pos.y as usize][pos.x as usize] = true;
        }
    }

    fn blocks_sight(terrain: &TerrainGrid, pos: Coord) -> bool {
        match terrain.get(pos) {
            Some(Feature::Wall) | Some(Feature::ClosedDoor) => true,
            None => true,
            _ => false,
        }
    }

    /// Recursive shadowcasting for one octant.
    fn cast_light(
        &mut self,
        terrain: &TerrainGrid,
        origin: Coord,
        radius: i32,
        row: i32,
        mut start_slope: f64,
        end_slope: f64,
        octant: u8,
    ) {
        if start_slope < end_slope { return }

        let mut blocked = false;
        let mut next_start_slope = start_slope;

        for j in row..=radius {
            if blocked { break }

            for dx in (-j)..=0 {
                let dy = -j;
                let (mx, my) = transform_octant(dx, dy, octant);
                let pos = Coord::new(origin.x + mx, origin.y + my);

                let left_slope = (dx as f64 - 0.5) / (dy as f64 + 0.5);
                let right_slope = (dx as f64 + 0.5) / (dy as f64 - 0.5);

                if start_slope < right_slope { continue }
                if end_slope > left_slope { break }

                // Check if in radius
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= radius * radius {
                    self.set_visible(pos);
                }

                if blocked {
                    if Self::blocks_sight(terrain, pos) {
                        next_start_slope = right_slope;
                    } else {
                        blocked = false;
                        start_slope = next_start_slope;
                    }
                } else if Self::blocks_sight(terrain, pos) && j < radius {
                    blocked = true;
                    self.cast_light(terrain, origin, radius, j + 1, start_slope, left_slope, octant);
                    next_start_slope = right_slope;
                }
            }
        }
    }
}

/// Transform coordinates from octant 0 to the specified octant.
fn transform_octant(col: i32, row: i32, octant: u8) -> (i32, i32) {
    match octant {
        0 => (col, row),
        1 => (row, col),
        2 => (row, -col),
        3 => (col, -row),
        4 => (-col, -row),
        5 => (-row, -col),
        6 => (-row, col),
        7 => (-col, row),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain;

    #[test]
    fn origin_always_visible() {
        let grid = terrain::hardcoded_dungeon();
        let mut vis = VisibilityMap::default();
        let origin = Coord::new(5, 5);
        vis.calculate(origin, 8, &grid);
        assert!(vis.is_visible(origin));
        assert!(vis.is_explored(origin));
    }

    #[test]
    fn wall_blocks_vision() {
        let grid = terrain::hardcoded_dungeon();
        let mut vis = VisibilityMap::default();
        // Player at (5,5) in room 1, wall at y=1
        vis.calculate(Coord::new(5, 5), 8, &grid);
        // Inside room should be visible
        assert!(vis.is_visible(Coord::new(7, 5)));
        // Outside the room walls should not be visible (behind walls)
        // The wall itself should be visible but beyond it should not
        assert!(!vis.is_visible(Coord::new(5, 0))); // Above room 1 walls
    }

    #[test]
    fn explored_persists() {
        let grid = terrain::hardcoded_dungeon();
        let mut vis = VisibilityMap::default();
        vis.calculate(Coord::new(5, 5), 8, &grid);
        assert!(vis.is_explored(Coord::new(7, 5)));

        // Recalculate from different position
        vis.calculate(Coord::new(25, 5), 8, &grid);
        // Old position no longer visible but still explored
        assert!(!vis.is_visible(Coord::new(5, 5)));
        assert!(vis.is_explored(Coord::new(7, 5)));
    }
}
