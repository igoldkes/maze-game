//! Thick wall rendering: black fill, white outline (outline thickness = wall_thickness / 20).
//! Wall thickness matches the walkable “path width” used for the player (see [`path_width`] = player diameter).

use crate::maze::Walls;

use macroquad::prelude::*;

/// Corridor width between opposing wall faces ≈ `cell_size - path_width()`; wall thickness equals this path width.
#[inline]
pub fn path_width(cell_size: f32) -> f32 {
    // Wider corridors than previous setting to improve readability in the main view.
    cell_size * 0.26
}

#[inline]
pub fn outline_thickness(wall_thickness: f32) -> f32 {
    (wall_thickness / 20.0).max(0.5)
}

pub struct FloorDraw {
    pub origin: Vec2,
    pub cell_size: f32,
    pub cols: usize,
    pub rows: usize,
    pub floor_color: Color,
    pub exit_cell: (usize, usize),
    pub show_exit_tint: bool,
}

/// Pass 1: floor tiles only.
pub fn draw_maze_floors(p: &FloorDraw) {
    for y in 0..p.rows {
        for x in 0..p.cols {
            let px = p.origin.x + x as f32 * p.cell_size;
            let py = p.origin.y + y as f32 * p.cell_size;
            let mut c = p.floor_color;
            if p.show_exit_tint && (x, y) == p.exit_cell {
                c = Color::from_rgba(90, 50, 55, 255);
            }
            draw_rectangle(px, py, p.cell_size, p.cell_size, c);
        }
    }
}

/// Pass 2: thick wall segments (orthogonal maze edges). `wall_thickness` == [`path_width`].
pub fn draw_maze_walls(
    origin: Vec2,
    cell_size: f32,
    cols: usize,
    rows: usize,
    bits_at: impl Fn(usize, usize) -> u8,
) {
    let wt = path_width(cell_size);
    let edge = outline_thickness(wt);
    let fill = Color::from_rgba(0, 0, 0, 255);
    let stroke = Color::from_rgba(255, 255, 255, 255);

    for y in 0..rows {
        for x in 0..cols {
            let px = origin.x + x as f32 * cell_size;
            let py = origin.y + y as f32 * cell_size;
            let bits = bits_at(x, y);

            if bits & Walls::NORTH != 0 {
                let wy = py - wt * 0.5;
                draw_black_wall_rect(px, wy, cell_size, wt, fill);
                draw_rectangle_lines(px, wy, cell_size, wt, edge, stroke);
            }
            if bits & Walls::SOUTH != 0 {
                let wy = py + cell_size - wt * 0.5;
                draw_black_wall_rect(px, wy, cell_size, wt, fill);
                draw_rectangle_lines(px, wy, cell_size, wt, edge, stroke);
            }
            if bits & Walls::WEST != 0 {
                let wx = px - wt * 0.5;
                draw_black_wall_rect(wx, py, wt, cell_size, fill);
                draw_rectangle_lines(wx, py, wt, cell_size, edge, stroke);
            }
            if bits & Walls::EAST != 0 {
                let wx = px + cell_size - wt * 0.5;
                draw_black_wall_rect(wx, py, wt, cell_size, fill);
                draw_rectangle_lines(wx, py, wt, cell_size, edge, stroke);
            }
        }
    }
}

fn draw_black_wall_rect(x: f32, y: f32, w: f32, h: f32, fill: Color) {
    draw_rectangle(x, y, w, h, fill);
}
