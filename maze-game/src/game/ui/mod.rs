//! Shared UI primitives: theme tokens, layout, and reusable components.
//!
//! ## Architecture
//!
//! - [`theme`] — spacing, type scale, palettes (`standard` / `high_contrast`), motion prefs.
//! - [`layout`] — responsive scale, safe margins, centered rects (reference height 720p).
//! - [`components`] — modal chrome, loading/error states, keyboard hints, wrapped text.
//!
//! Macroquad is immediate-mode: there is no DOM, ARIA, or built-in screen reader. Treat
//! **keyboard-first** design, **readable type floors**, **visible focus**, and **high contrast**
//! as your accessibility baseline; wire OS narration later via platform crates if needed.

use macroquad::prelude::*;

pub mod components;
pub mod layout;
pub mod theme;

pub struct PanelStyle {
    pub bg: Color,
    pub border: Option<(f32, Color)>,
}

pub fn draw_panel(rect: Rect, style: PanelStyle) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, style.bg);
    if let Some((thickness, color)) = style.border {
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, thickness, color);
    }
}

/// Solid black fill in **screen space** (default camera). Use before drawing intro UI on top.
#[inline]
pub fn draw_fullscreen_opaque_black() {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(0, 0, 0, 255),
    );
}
