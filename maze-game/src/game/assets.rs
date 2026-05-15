//! In-game textures. For file-based loading, see Macroquad’s upstream `examples/texture.rs`
//! ([macroquad texture example](https://github.com/not-fl3/macroquad/blob/master/examples/texture.rs)):
//! `load_texture("path.png").await`, then `draw_texture` / `draw_texture_ex` with [`DrawTextureParams`].

use macroquad::prelude::*;
use std::fs;

/// NOTE:
/// This is the current player avatar generator! 
///
/// Bird’s-eye “hat” silhouette: wide brim + crown (top-down). Generated at runtime; swap for a PNG via
/// [`Texture2D::from_file_with_format`] or [`load_texture`].
pub fn build_player_hat_texture() -> Texture2D {
    const W: u32 = 64;
    const H: u32 = 64;
    let mut img = Image::gen_image_color(W as u16, H as u16, Color::from_rgba(0, 0, 0, 0));
    let cx = W as f32 * 0.5;
    let cy = H as f32 * 0.48;
    let brim_a = 24.0_f32;
    let brim_b = 9.0_f32;
    let crown_cx = cx;
    let crown_cy = cy - 10.0;
    let crown_a = 12.0_f32;
    let crown_b = 14.0_f32;
    // Burnt orange (#B7410E) for stronger visibility against dark maze floors.
    let fill = Color::from_rgba(183, 65, 14, 255);

    for y in 0..H {
        for x in 0..W {
            let fx = x as f32;
            let fy = y as f32;
            let in_brim = ellipse_contains(fx, fy, cx, cy + 6.0, brim_a, brim_b);
            let in_crown = ellipse_contains(fx, fy, crown_cx, crown_cy, crown_a, crown_b);
            if in_brim || in_crown {
                img.set_pixel(x, y, fill);
            }
        }
    }

    Texture2D::from_image(&img)
}

/// Load an exit star icon from disk (`assets/mazeexitstar.png` in this project).
/// Falls back to a generated yellow star-like marker when the file is absent.
pub fn load_exit_star_texture(path: &str) -> Texture2D {
    match fs::read(path) {
        Ok(bytes) => Texture2D::from_file_with_format(&bytes, None),
        Err(_) => build_fallback_star_texture(),
    }
}

/// Load hint item icon from disk (`assets/hintitem.png`).
/// Falls back to a cyan diamond marker when unavailable.
pub fn load_hint_item_texture(path: &str) -> Texture2D {
    match fs::read(path) {
        Ok(bytes) => Texture2D::from_file_with_format(&bytes, None),
        Err(_) => build_fallback_hint_texture(),
    }
}

/// Load old-paper image for story/map UI (`assets/mapimage.png`).
/// Falls back to a generated parchment-like texture.
pub fn load_map_paper_texture(path: &str) -> Texture2D {
    match fs::read(path) {
        Ok(bytes) => Texture2D::from_file_with_format(&bytes, None),
        Err(_) => build_fallback_map_paper_texture(),
    }
}

fn build_fallback_star_texture() -> Texture2D {
    const W: u32 = 64;
    const H: u32 = 64;
    let mut img = Image::gen_image_color(W as u16, H as u16, Color::from_rgba(0, 0, 0, 0));
    let cx = W as f32 * 0.5;
    let cy = H as f32 * 0.5;
    let gold = Color::from_rgba(255, 216, 64, 255);
    let edge = Color::from_rgba(255, 245, 180, 255);
    for y in 0..H {
        for x in 0..W {
            let dx = (x as f32 - cx).abs();
            let dy = (y as f32 - cy).abs();
            // Simple 8-point star from cross + diagonals.
            let cross = dx <= 5.0 || dy <= 5.0;
            let diag = (dx - dy).abs() <= 3.0;
            let core = (dx * dx + dy * dy) <= 12.0 * 12.0;
            if cross || diag || core {
                let c = if (dx + dy) > 18.0 { edge } else { gold };
                img.set_pixel(x, y, c);
            }
        }
    }
    Texture2D::from_image(&img)
}

fn build_fallback_hint_texture() -> Texture2D {
    const W: u32 = 64;
    const H: u32 = 64;
    let mut img = Image::gen_image_color(W as u16, H as u16, Color::from_rgba(0, 0, 0, 0));
    let cx = W as f32 * 0.5;
    let cy = H as f32 * 0.5;
    let fill = Color::from_rgba(90, 230, 255, 255);
    let edge = Color::from_rgba(210, 250, 255, 255);
    for y in 0..H {
        for x in 0..W {
            let dx = (x as f32 - cx).abs();
            let dy = (y as f32 - cy).abs();
            if dx + dy <= 18.0 {
                let c = if dx + dy >= 15.0 { edge } else { fill };
                img.set_pixel(x, y, c);
            }
        }
    }
    Texture2D::from_image(&img)
}

fn build_fallback_map_paper_texture() -> Texture2D {
    const W: u32 = 420;
    const H: u32 = 300;
    let mut img = Image::gen_image_color(W as u16, H as u16, Color::from_rgba(0, 0, 0, 0));
    let base = Color::from_rgba(212, 188, 142, 255);
    let dark = Color::from_rgba(178, 150, 110, 255);
    for y in 0..H {
        for x in 0..W {
            let edge = x.min(W - 1 - x).min(y.min(H - 1 - y)) as f32;
            let t = (edge / 28.0).clamp(0.0, 1.0);
            let r = dark.r + (base.r - dark.r) * t;
            let g = dark.g + (base.g - dark.g) * t;
            let b = dark.b + (base.b - dark.b) * t;
            img.set_pixel(x, y, Color::new(r, g, b, 1.0));
        }
    }
    Texture2D::from_image(&img)
}

fn ellipse_contains(x: f32, y: f32, cx: f32, cy: f32, a: f32, b: f32) -> bool {
    if a <= 0.0 || b <= 0.0 {
        return false;
    }
    let nx = (x - cx) / a;
    let ny = (y - cy) / b;
    nx * nx + ny * ny <= 1.0
}
