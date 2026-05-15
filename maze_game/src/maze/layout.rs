//! Start / exit placement on a rectangular orthogonal maze.

use super::bfs_distances;
use super::is_reachable;
use super::Maze;

/// Approximate **center** cell (integer indices). For even sizes, picks the cell left/up of the geometric center.
pub fn center_cell(width: usize, height: usize) -> (usize, usize) {
    ((width.saturating_sub(1)) / 2, (height.saturating_sub(1)) / 2)
}

fn is_perimeter(width: usize, height: usize, x: usize, y: usize) -> bool {
    x == 0 || y == 0 || x + 1 == width || y + 1 == height
}

/// Pick a **single exit** on the outer edge: among perimeter cells, choose one **farthest** from `start`
/// (by maze graph distance). Skips `start` if it lies on the perimeter.
pub fn exit_farthest_on_perimeter(maze: &Maze, start: (usize, usize)) -> (usize, usize) {
    let w = maze.width();
    let h = maze.height();
    let dist = bfs_distances(maze, start);
    let mut best: Option<(usize, (usize, usize))> = None;
    for y in 0..h {
        for x in 0..w {
            if !is_perimeter(w, h, x, y) {
                continue;
            }
            if (x, y) == start {
                continue;
            }
            let d = dist[y * w + x];
            if d == usize::MAX {
                continue;
            }
            match best {
                None => best = Some((d, (x, y))),
                Some((bd, _)) if d > bd => best = Some((d, (x, y))),
                _ => {}
            }
        }
    }
    if let Some((_, c)) = best {
        debug_assert!(is_reachable(maze, start, c));
        return c;
    }
    // Degenerate tiny maze: fall back to any corner different from start.
    if w > 1 {
        return (w - 1, start.1.min(h - 1));
    }
    if h > 1 {
        return (start.0.min(w - 1), h - 1);
    }
    (0, 0)
}
