//! Visibility helpers for world rendering.
//! Precomputes per-frame visible cells so draw code can reuse visibility checks.

pub fn visibility_grid(
    cols: usize,
    rows: usize,
    center: (usize, usize),
    radius_cells: f32,
) -> Vec<bool> {
    let mut vis = vec![false; cols * rows];
    let r2 = radius_cells * radius_cells;
    for y in 0..rows {
        for x in 0..cols {
            let dx = x as f32 - center.0 as f32;
            let dy = y as f32 - center.1 as f32;
            vis[y * cols + x] = dx * dx + dy * dy <= r2;
        }
    }
    vis
}

#[inline]
pub fn is_visible(vis: &[bool], cols: usize, x: usize, y: usize) -> bool {
    vis[y * cols + x]
}
