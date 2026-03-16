//! A* pathfinding and BFS exploration for grid-based movement.

use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::cmp::Ordering;
use crate::terrain::TerrainGrid;
use crate::monster::MonsterGrid;
use crate::types::*;

const MAX_SEARCH: i32 = 20;

#[derive(Eq, PartialEq)]
struct Node {
    pos: Coord,
    cost: i32,     // g: actual cost from start
    priority: i32, // f: cost + heuristic
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority) // min-heap
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn heuristic(a: Coord, b: Coord) -> i32 {
    // Chebyshev distance (8-directional movement)
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

const DIRS: [(i32, i32); 8] = [
    (-1, -1), (0, -1), (1, -1),
    (-1, 0),           (1, 0),
    (-1, 1),  (0, 1),  (1, 1),
];

/// A* pathfinding: returns the next step from `from` toward `to`.
/// Avoids impassable terrain and tiles occupied by other monsters.
/// `target_occupied_ok` should be true when pathfinding TO the player (final tile is the target).
pub fn astar_next_step(
    from: Coord,
    to: Coord,
    terrain: &TerrainGrid,
    monster_grid: &MonsterGrid,
) -> Option<Coord> {
    if from == to { return None }
    if heuristic(from, to) > MAX_SEARCH { return None }

    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut g_score: HashMap<(i32, i32), i32> = HashMap::new();

    let start = (from.x, from.y);
    let goal = (to.x, to.y);

    g_score.insert(start, 0);
    open.push(Node { pos: from, cost: 0, priority: heuristic(from, to) });

    while let Some(current) = open.pop() {
        let cur = (current.pos.x, current.pos.y);
        if cur == goal {
            // Reconstruct path and return first step
            let mut step = cur;
            while let Some(&prev) = came_from.get(&step) {
                if prev == start { return Some(Coord::new(step.0, step.1)) }
                step = prev;
            }
            return Some(Coord::new(step.0, step.1));
        }

        let cur_cost = *g_score.get(&cur).unwrap_or(&i32::MAX);
        if current.cost > cur_cost { continue } // outdated entry

        for (dx, dy) in DIRS {
            let nx = cur.0 + dx;
            let ny = cur.1 + dy;
            let next = Coord::new(nx, ny);

            if !next.in_bounds() { continue }
            if !terrain.is_passable(next) { continue }

            // Allow moving TO the goal even if occupied (it's the target)
            let nk = (nx, ny);
            if nk != goal && monster_grid.get(next).is_some() { continue }

            let new_cost = cur_cost + 1;
            if new_cost < *g_score.get(&nk).unwrap_or(&i32::MAX) {
                g_score.insert(nk, new_cost);
                came_from.insert(nk, cur);
                open.push(Node {
                    pos: next,
                    cost: new_cost,
                    priority: new_cost + heuristic(next, Coord::new(goal.0, goal.1)),
                });
            }
        }
    }

    None // no path found
}

/// BFS to find the nearest unexplored reachable tile.
/// Returns the first step toward it (using A* for the actual path).
pub fn nearest_unexplored(
    from: Coord,
    terrain: &TerrainGrid,
    monster_grid: &MonsterGrid,
    explored: &[[bool; MAP_WIDTH]; MAP_HEIGHT],
) -> Option<Coord> {
    let mut visited = [[false; MAP_WIDTH]; MAP_HEIGHT];
    let mut queue = VecDeque::new();

    visited[from.y as usize][from.x as usize] = true;
    queue.push_back(from);

    while let Some(pos) = queue.pop_front() {
        // Check if this tile borders an unexplored tile
        for (dx, dy) in DIRS {
            let neighbor = Coord::new(pos.x + dx, pos.y + dy);
            if !neighbor.in_bounds() { continue }
            if !explored[neighbor.y as usize][neighbor.x as usize] {
                // Found an unexplored neighbor — pathfind to `pos` (the explored tile next to it)
                if pos == from { return Some(neighbor) }
                return astar_next_step(from, pos, terrain, monster_grid);
            }
        }

        // Expand BFS
        for (dx, dy) in DIRS {
            let next = Coord::new(pos.x + dx, pos.y + dy);
            if !next.in_bounds() { continue }
            if visited[next.y as usize][next.x as usize] { continue }
            if !terrain.is_passable(next) { continue }
            if monster_grid.get(next).is_some() { continue }
            visited[next.y as usize][next.x as usize] = true;
            queue.push_back(next);
        }
    }

    None // fully explored or no reachable unexplored tiles
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain;

    #[test]
    fn direct_path() {
        let grid = terrain::hardcoded_dungeon();
        let mgrid = MonsterGrid::default();
        // Room 1: (2,2)-(12,8), start at (3,5), target at (10,5)
        let step = astar_next_step(Coord::new(3, 5), Coord::new(10, 5), &grid, &mgrid);
        assert!(step.is_some(), "should find a step");
        let s = step.unwrap();
        // Step should move closer to target (x increases)
        assert!(s.x >= 4, "should move toward target (x >= 4), got {:?}", s);
    }

    #[test]
    fn navigate_around_wall() {
        // Create a small grid with an L-shaped wall
        let mut cells = [[terrain::Feature::Floor; MAP_WIDTH]; MAP_HEIGHT];
        // Wall from (5,3) to (5,7)
        for y in 3..=7 { cells[y][5] = terrain::Feature::Wall; }
        // Wall from (5,7) to (8,7)
        for x in 5..=8 { cells[7][x] = terrain::Feature::Wall; }
        let grid = terrain::TerrainGrid { cells };
        let mgrid = MonsterGrid::default();

        // Monster at (3,5) wants to reach (8,5) — must go around the wall
        let step = astar_next_step(Coord::new(3, 5), Coord::new(8, 5), &grid, &mgrid);
        assert!(step.is_some(), "should find a path around the wall");
        // First step should NOT be directly right (that would hit the wall at x=5)
        let s = step.unwrap();
        assert!(s.x != 5 || s.y != 5, "should not walk into wall");
    }

    #[test]
    fn no_path() {
        // Completely walled off
        let mut cells = [[terrain::Feature::Wall; MAP_WIDTH]; MAP_HEIGHT];
        cells[5][5] = terrain::Feature::Floor;
        cells[10][10] = terrain::Feature::Floor;
        let grid = terrain::TerrainGrid { cells };
        let mgrid = MonsterGrid::default();

        let step = astar_next_step(Coord::new(5, 5), Coord::new(10, 10), &grid, &mgrid);
        assert!(step.is_none());
    }

    #[test]
    fn bfs_finds_unexplored() {
        let grid = terrain::hardcoded_dungeon();
        let mgrid = MonsterGrid::default();
        // Mark everything as explored except the corridor
        let mut explored = [[true; MAP_WIDTH]; MAP_HEIGHT];
        for x in 13..=19 { explored[5][x] = false; } // corridor is unexplored

        let step = nearest_unexplored(Coord::new(5, 5), &grid, &mgrid, &explored);
        assert!(step.is_some(), "should find the unexplored corridor");
    }
}
