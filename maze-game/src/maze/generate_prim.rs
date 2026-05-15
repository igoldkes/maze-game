//! Randomized Prim / "growing tree" style carve on a grid.
//! Compared to recursive backtracking (often longer corridors), Prim tends to produce more uniform
//! branching—similar to many orthogonal mazes from web tools like [mazegenerator.net](https://www.mazegenerator.net/).

use super::{Maze, Walls};

use macroquad::rand::gen_range;

type Edge = ((usize, usize), (usize, usize));

const DX: [i32; 4] = [0, 1, 0, -1];
const DY: [i32; 4] = [-1, 0, 1, 0];

fn remove_wall_between(cells: &mut [u8], width: usize, a: (usize, usize), b: (usize, usize)) {
    let (ax, ay) = a;
    let (bx, by) = b;
    if bx == ax + 1 && by == ay {
        cells[ay * width + ax] &= !Walls::EAST;
        cells[by * width + bx] &= !Walls::WEST;
    } else if ax == bx + 1 && by == ay {
        cells[ay * width + ax] &= !Walls::WEST;
        cells[by * width + bx] &= !Walls::EAST;
    } else if by == ay + 1 && bx == ax {
        cells[ay * width + ax] &= !Walls::SOUTH;
        cells[by * width + bx] &= !Walls::NORTH;
    } else if ay == by + 1 && bx == ax {
        cells[ay * width + ax] &= !Walls::NORTH;
        cells[by * width + bx] &= !Walls::SOUTH;
    } else {
        debug_assert!(false, "non-adjacent cells");
    }
}

fn push_frontier_edges(
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    in_maze: &[bool],
    edges: &mut Vec<Edge>,
) {
    for d in 0..4 {
        let nx = x as i32 + DX[d];
        let ny = y as i32 + DY[d];
        if nx < 0 || ny < 0 {
            continue;
        }
        let nx = nx as usize;
        let ny = ny as usize;
        if nx >= width || ny >= height {
            continue;
        }
        let n_idx = ny * width + nx;
        if !in_maze[n_idx] {
            edges.push(((x, y), (nx, ny)));
        }
    }
}

pub fn generate_randomized_prim(width: usize, height: usize) -> Maze {
    assert!(width >= 1 && height >= 1);
    let mut cells = vec![Walls::ALL; width * height];
    let mut in_maze = vec![false; width * height];

    let sx = gen_range(0, width);
    let sy = gen_range(0, height);
    in_maze[sy * width + sx] = true;

    let mut edges: Vec<Edge> = Vec::new();
    push_frontier_edges(width, height, sx, sy, &in_maze, &mut edges);

    while !edges.is_empty() {
        let i = gen_range(0, edges.len());
        let ((ax, ay), (bx, by)) = edges.swap_remove(i);
        let ai = ay * width + ax;
        let bi = by * width + bx;
        let a_in = in_maze[ai];
        let b_in = in_maze[bi];
        if a_in == b_in {
            continue;
        }
        let (new_x, new_y) = if a_in { (bx, by) } else { (ax, ay) };
        remove_wall_between(&mut cells, width, (ax, ay), (bx, by));
        let ni = new_y * width + new_x;
        if in_maze[ni] {
            continue;
        }
        in_maze[ni] = true;
        push_frontier_edges(width, height, new_x, new_y, &in_maze, &mut edges);
    }

    Maze::new(width, height, cells)
}
