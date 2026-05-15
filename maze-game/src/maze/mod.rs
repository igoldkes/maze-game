//! Grid maze: each cell stores four wall bits (north, east, south, west). Passages are **absent** walls.
//!
//! ## Compared to typical web generators (e.g. [mazegenerator.net](https://www.mazegenerator.net/))
//! - This project uses **orthogonal** square cells—the same family as “Orthogonal (Square cells)” on that site.
//! - We implement two classic **perfect maze** generators (spanning trees of the grid graph):
//!   **recursive backtracking** (depth-first; longer corridors) and **randomized Prim** (often shorter
//!   branches, closer to many published “Prim” style mazes). Shape presets (circular / hex / triangle)
//!   would need different tilings and are not implemented here.

mod mazedataaccess;
mod generate;
mod generate_prim;
mod layout;
mod solvability;

pub use layout::{center_cell, exit_farthest_on_perimeter};
pub use solvability::{bfs_distances, is_reachable, shortest_path_len};

/// Wall bits on a single cell. `1` means a solid wall.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Walls(pub u8);

impl Walls {
    pub const NORTH: u8 = 1 << 0;
    pub const EAST: u8 = 1 << 1;
    pub const SOUTH: u8 = 1 << 2;
    pub const WEST: u8 = 1 << 3;
    pub const ALL: u8 = Self::NORTH | Self::EAST | Self::SOUTH | Self::WEST;
}

/// Rectangular perfect maze: `width` × `height` cells, row-major in `cells`.
#[derive(Clone, Debug)]
pub struct Maze {
    width: usize,
    height: usize,
    cells: Vec<u8>,
}

impl Maze {
    pub fn new(width: usize, height: usize, cells: Vec<u8>) -> Self {
        debug_assert_eq!(cells.len(), width * height);
        Self {
            width,
            height,
            cells,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn walls(&self, x: usize, y: usize) -> Walls {
        Walls(self.cells[y * self.width + x])
    }

    /// Default: **randomized Prim** (often shorter branches; similar to many “Prim” mazes online).
    pub fn generate(width: usize, height: usize) -> Self {
        let m = generate_prim::generate_randomized_prim(width, height);
        debug_assert!(
            is_fully_connected(&m),
            "generator must yield one connected component"
        );
        m
    }

    /// **Recursive backtracking** (depth-first): tends to produce longer straight corridors than [`Maze::generate`].
    pub fn generate_depth_first(width: usize, height: usize) -> Self {
        let m = generate::generate_recursive_backtracking(width, height);
        debug_assert!(
            is_fully_connected(&m),
            "generator must yield one connected component"
        );
        m
    }
}

fn is_fully_connected(maze: &Maze) -> bool {
    let w = maze.width();
    let h = maze.height();
    if w == 0 || h == 0 {
        return false;
    }
    let d = bfs_distances(maze, (0, 0));
    d.iter().all(|&x| x != usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_maze_fully_connected() {
        for make in [Maze::generate, Maze::generate_depth_first] {
            for w in [1usize, 2, 5, 15, 31] {
                for h in [1usize, 2, 5, 12, 21] {
                    let m = make(w, h);
                    assert!(is_fully_connected(&m), "{}×{}", w, h);
                }
            }
        }
    }

    #[test]
    fn start_to_exit_reachable() {
        let m = Maze::generate(21, 15);
        let s = center_cell(21, 15);
        let e = exit_farthest_on_perimeter(&m, s);
        assert!(is_reachable(&m, s, e));
    }
}
