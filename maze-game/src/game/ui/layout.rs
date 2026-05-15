//! Responsive layout: safe margins, scaled typography, centered panels.

use super::theme::{TypeScale, REFERENCE_SCREEN_H};
use macroquad::prelude::*;

/// Scale UI relative to reference 720p height (clamped so tiny windows stay usable).
pub fn ui_scale() -> f32 {
    (screen_height() / REFERENCE_SCREEN_H).clamp(0.72, 1.35)
}

pub fn scaled_type(base: &TypeScale, scale: f32) -> TypeScale {
    TypeScale {
        body_min: (base.body_min * scale).max(14.0),
        body: (base.body * scale).max(base.body_min),
        title: base.title * scale,
        headline: base.headline * scale,
    }
}

/// Horizontal and vertical margin from screen edges (scaled).
pub fn safe_margins(scale: f32) -> f32 {
    (24.0 * scale).max(16.0)
}

/// Centered rectangle with width/height clamped to safe area.
pub fn centered_clamped_rect(preferred_w: f32, preferred_h: f32, margin: f32) -> Rect {
    let w = screen_width();
    let h = screen_height();
    let max_w = (w - margin * 2.0).max(120.0);
    let max_h = (h - margin * 2.0).max(80.0);
    let pw = preferred_w.min(max_w);
    let ph = preferred_h.min(max_h);
    let x = (w - pw) * 0.5;
    let y = (h - ph) * 0.5;
    Rect::new(x, y, pw, ph)
}

/// Stack helper: next Y after a line of text (Macroquad `draw_text` uses baseline-ish y).
pub fn line_height(font_size: f32) -> f32 {
    font_size * 1.35
}
