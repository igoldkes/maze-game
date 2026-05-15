//! Reachability/path helpers (BFS) used for validation, exit placement, and debug metrics.

use super::Maze;
use super::Walls;

use std::collections::VecDeque;

/// Breadth-first search on the maze graph (cells are nodes; edges exist through absent walls).
pub fn is_reachable(maze: &Maze, start: (usize, usize), goal: (usize, usize)) -> bool {
    shortest_path_len(maze, start, goal).is_some()
}

/// Number of **steps** (cell-to-cell moves) from `start` to `goal`, or `None` if unreachable.
pub fn shortest_path_len(maze: &Maze, start: (usize, usize), goal: (usize, usize)) -> Option<usize> {
    let w = maze.width();
    let h = maze.height();
    if start.0 >= w || start.1 >= h || goal.0 >= w || goal.1 >= h {
        return None;
    }
    if start == goal {
        return Some(0);
    }

    let mut dist = vec![usize::MAX; w * h];
    let si = start.1 * w + start.0;
    dist[si] = 0;
    let mut q = VecDeque::new();
    q.push_back(start);

    while let Some((x, y)) = q.pop_front() {
        let i = y * w + x;
        let d = dist[i];
        let bits = maze.walls(x, y).0;

        if bits & Walls::NORTH == 0 && y > 0 {
            let ni = (y - 1) * w + x;
            if dist[ni] == usize::MAX {
                dist[ni] = d + 1;
                if (x, y - 1) == goal {
                    return Some(d + 1);
                }
                q.push_back((x, y - 1));
            }
        }
        if bits & Walls::SOUTH == 0 && y + 1 < h {
            let ni = (y + 1) * w + x;
            if dist[ni] == usize::MAX {
                dist[ni] = d + 1;
                if (x, y + 1) == goal {
                    return Some(d + 1);
                }
                q.push_back((x, y + 1));
            }
        }
        if bits & Walls::WEST == 0 && x > 0 {
            let ni = y * w + (x - 1);
            if dist[ni] == usize::MAX {
                dist[ni] = d + 1;
                if (x - 1, y) == goal {
                    return Some(d + 1);
                }
                q.push_back((x - 1, y));
            }
        }
        if bits & Walls::EAST == 0 && x + 1 < w {
            let ni = y * w + (x + 1);
            if dist[ni] == usize::MAX {
                dist[ni] = d + 1;
                if (x + 1, y) == goal {
                    return Some(d + 1);
                }
                q.push_back((x + 1, y));
            }
        }
    }

    None
}

/// Shortest-path distance from `start` to every cell; `usize::MAX` if unreachable.
pub fn bfs_distances(maze: &Maze, start: (usize, usize)) -> Vec<usize> {
    let w = maze.width();
    let h = maze.height();
    let mut dist = vec![usize::MAX; w * h];
    if start.0 >= w || start.1 >= h {
        return dist;
    }
    let si = start.1 * w + start.0;
    dist[si] = 0;
    let mut q = VecDeque::new();
    q.push_back(start);

    while let Some((x, y)) = q.pop_front() {
        let i = y * w + x;
        let d = dist[i];
        let bits = maze.walls(x, y).0;

        if bits & Walls::NORTH == 0 && y > 0 {
            let ni = (y - 1) * w + x;
            if dist[ni] == usize::MAX {
                dist[ni] = d + 1;
                q.push_back((x, y - 1));
            }
        }
        if bits & Walls::SOUTH == 0 && y + 1 < h {
            let ni = (y + 1) * w + x;
            if dist[ni] == usize::MAX {
                dist[ni] = d + 1;
                q.push_back((x, y + 1));
            }
        }
        if bits & Walls::WEST == 0 && x > 0 {
            let ni = y * w + (x - 1);
            if dist[ni] == usize::MAX {
                dist[ni] = d + 1;
                q.push_back((x - 1, y));
            }
        }
        if bits & Walls::EAST == 0 && x + 1 < w {
            let ni = y * w + (x + 1);
            if dist[ni] == usize::MAX {
                dist[ni] = d + 1;
                q.push_back((x + 1, y));
            }
        }
    }

    dist
}
