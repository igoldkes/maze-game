//! Recursive-backtracking (depth-first) maze generation.

use super::{Maze, Walls};

use macroquad::rand::gen_range;

/// Direction index: 0 = north, 1 = east, 2 = south, 3 = west.
const DX: [i32; 4] = [0, 1, 0, -1];
const DY: [i32; 4] = [-1, 0, 1, 0];

fn wall_masks_for_direction(dir: usize) -> (u8, u8) {
    match dir {
        0 => (Walls::NORTH, Walls::SOUTH),
        1 => (Walls::EAST, Walls::WEST),
        2 => (Walls::SOUTH, Walls::NORTH),
        3 => (Walls::WEST, Walls::EAST),
        _ => unreachable!(),
    }
}

fn shuffle_dirs() -> [usize; 4] {
    let mut d = [0usize, 1, 2, 3];
    for i in (1..4).rev() {
        let j = gen_range(0, i + 1);
        d.swap(i, j);
    }
    d
}

pub fn generate_recursive_backtracking(width: usize, height: usize) -> Maze {
    assert!(width >= 1 && height >= 1);
    let mut cells = vec![Walls::ALL; width * height];
    let mut visited = vec![false; width * height];
    let mut stack: Vec<(usize, usize)> = Vec::new();

    stack.push((0, 0));
    visited[0] = true;

    while !stack.is_empty() {
        let (x, y) = *stack.last().expect("stack non-empty");
        let order = shuffle_dirs();
        let mut carved = false;

        for dir in order {
            let nx = x as i32 + DX[dir];
            let ny = y as i32 + DY[dir];
            if nx < 0 || ny < 0 {
                continue;
            }
            let nx = nx as usize;
            let ny = ny as usize;
            if nx >= width || ny >= height {
                continue;
            }
            let ni = ny * width + nx;
            if visited[ni] {
                continue;
            }

            let (here, there) = wall_masks_for_direction(dir);
            cells[y * width + x] &= !here;
            cells[ni] &= !there;

            visited[ni] = true;
            stack.push((nx, ny));
            carved = true;
            break;
        }

        if !carved {
            stack.pop();
        }
    }

    Maze::new(width, height, cells)
}
