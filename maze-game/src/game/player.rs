//! Grid-locked player movement (cell-to-cell).
//! Cardinal only, no diagonal movement. This keeps traversal aligned with maze blocks.
//!
//! Compass (Macroquad Y grows **down**): North = -Y, South = +Y, West = -X, East = +X.
//! Keys: WASD or arrow keys.

use crate::maze::{Maze, Walls};

use macroquad::prelude::*;
use macroquad::audio::{load_sound, play_sound_once, stop_sound, Sound};
use macroquad::rand::gen_range;

/// Player in world space. Uses a **bird’s-eye** hat texture (see `assets.rs` / `assets/`).
pub struct Player {
    pub pos: Vec2,
    pub radius: f32,
    /// Cell transition speed in cells/second.
    pub speed: f32,
    pub hat_texture: Texture2D,
    cell: (usize, usize),
    target_cell: (usize, usize),
    footsteps: Vec<Sound>,
    pub stage: usize,
    blocked: bool,
}

impl Player {
    pub async fn new_at_cell_center(
        origin: Vec2,
        cell_size: f32,
        cell_x: usize,
        cell_y: usize,
        hat_texture: Texture2D,
        stage: usize,
    ) -> Self {
        let footstep_1 = load_sound("assets/audio_assets/footstep_1.wav").await.unwrap();
        let footstep_2 = load_sound("assets/audio_assets/footstep_2.wav").await.unwrap();
        let footstep_3 = load_sound("assets/audio_assets/footstep_3.wav").await.unwrap();
        let footsteps = vec![footstep_1, footstep_2, footstep_3];

        let c = Self::cell_center(origin, cell_size, cell_x, cell_y);
        Self {
            pos: c,
            radius: cell_size * 0.22,
            speed: 5.0,
            hat_texture,
            cell: (cell_x, cell_y),
            target_cell: (cell_x, cell_y),
            footsteps,
            stage,
            blocked: false,
        }
    }

    pub fn cell_center(origin: Vec2, cell_size: f32, cell_x: usize, cell_y: usize) -> Vec2 {
        origin
            + vec2(
                cell_x as f32 * cell_size + cell_size * 0.5,
                cell_y as f32 * cell_size + cell_size * 0.5,
            )
    }

    pub fn update(&mut self, maze: &Maze, origin: Vec2, cell_size: f32, dt: f32) {
        // Acquire a new target only when fully centered in a cell.
        if self.cell == self.target_cell {
            if let Some(next) = self.next_cell_from_input(maze) {
                self.target_cell = next;
            }
        }

        let target_pos = Self::cell_center(origin, cell_size, self.target_cell.0, self.target_cell.1);
        let delta = target_pos - self.pos;
        let dist = delta.length();
        if dist <= f32::EPSILON {
            self.pos = target_pos;
            self.cell = self.target_cell;
            return;
        }

        let px_per_sec = self.speed * cell_size;
        let step = px_per_sec * dt;
        if step >= dist {
            self.pos = target_pos;
            self.cell = self.target_cell;
        } else {
            self.pos += delta / dist * step;
        }
    }

    pub fn draw(&self) {
        let s = self.radius * 2.0;
        draw_texture_ex(
            &self.hat_texture,
            self.pos.x - s * 0.5,
            self.pos.y - s * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(s, s)),
                ..Default::default()
            },
        );
        // Always draw a small solid marker so player remains visible even if texture filtering/scaling hides details.
        draw_circle(
            self.pos.x,
            self.pos.y,
            (self.radius * 23.0)/(1.0 + (self.stage as f32 - 1.0)/13.0),
            Color::from_rgba(255, 225, 75, 255),
        );
        for i in 1..23 {
            let val = 22-i;
            draw_circle(
                self.pos.x,
                self.pos.y,
                (self.radius * (val as f32)/1.5)/(1.0 + (self.stage as f32 - 1.0)/13.0),
                Color::from_rgba(255 - (val * 4), 238 - (val * 5), 160 - (val * 7), 255),
            );
        }
    }

    /// Cell indices from world position (clamped to the grid).
    pub fn current_cell(
        &self,
        _origin: Vec2,
        _cell_size: f32,
        _maze_w: usize,
        _maze_h: usize,
    ) -> (usize, usize) {
        self.cell
    }

    pub fn respawn_at_cell(&mut self, origin: Vec2, cell_size: f32, cell_x: usize, cell_y: usize) {
        self.cell = (cell_x, cell_y);
        self.target_cell = (cell_x, cell_y);
        self.pos = Self::cell_center(origin, cell_size, cell_x, cell_y);
    }

    fn next_cell_from_input(&mut self, maze: &Maze) -> Option<(usize, usize)> {
        let mut dx = 0_i32;
        let mut dy = 0_i32;
        // No diagonals: choose one direction, with vertical priority if both pressed.
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            let step = gen_range(0, 4);
            if step != 3 {
                if !self.blocked {
                    play_sound_once(&self.footsteps[step]);
                }
            }
            dy = -1;
        } else if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            let step = gen_range(0, 4);
            if step != 3 {
                if !self.blocked {
                    play_sound_once(&self.footsteps[step]);
                }
            }
            dy = 1;
        } else if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            let step = gen_range(0, 4);
            if step != 3 {
                if !self.blocked {
                    play_sound_once(&self.footsteps[step]);
                }
            }
            dx = 1;
        } else if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            let step = gen_range(0, 4);
            if step != 3 {
                if !self.blocked {
                    play_sound_once(&self.footsteps[step]);
                }
            }
            dx = -1;
        } else {
            return None;
        }

        let (x, y) = self.cell;
        let bits = maze.walls(x, y).0;
        let blocked = (dx, dy) == (0, -1) && bits & Walls::NORTH != 0
            || (dx, dy) == (0, 1) && bits & Walls::SOUTH != 0
            || (dx, dy) == (-1, 0) && bits & Walls::WEST != 0
            || (dx, dy) == (1, 0) && bits & Walls::EAST != 0;
        if blocked {
            self.blocked = true;
            return None;
        } else {
            self.blocked = false;
        }

        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx < 0 || ny < 0 || nx >= maze.width() as i32 || ny >= maze.height() as i32 {
            return None;
        }
        Some((nx as usize, ny as usize))
    }
}
