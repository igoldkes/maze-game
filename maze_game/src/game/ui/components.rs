#![allow(dead_code)]
// Public kit surface: loading states, layout helpers, etc. are used as screens adopt them.

//! Reusable UI components (props + draw functions) for Macroquad immediate-mode UI.
//!
//! # Example
//!
//! ```ignore
//! use crate::game::ui::components::{self, LoadingState, ModalChromeProps};
//! use crate::game::ui::layout;
//! use crate::game::ui::theme::{TypeScale, UiPreferences};
//!
//! let prefs = UiPreferences::default();
//! let scale = layout::ui_scale();
//! let ty = layout::scaled_type(&TypeScale::default(), scale);
//! let margin = layout::safe_margins(scale);
//! let rect = layout::centered_clamped_rect(760.0 * scale, 280.0 * scale, margin);
//! components::draw_modal_chrome(&ModalChromeProps {
//!     rect,
//!     title: Some("Settings"),
//!     palette: prefs.palette(),
//!     focused: true,
//!     semantic_id: "modal-settings",
//! });
//! components::draw_loading_state(
//!     LoadingState::Loading { label: "Saving…" },
//!     rect,
//!     &ty,
//!     &prefs,
//!     1.7,
//! );
//! ```

use super::layout::{self, line_height};
use super::theme::{TypeScale, UiPalette, UiPreferences};
use super::{draw_panel, PanelStyle};
use macroquad::prelude::*;

fn measure_w(text: &str, font_size: f32) -> f32 {
    measure_text(text, None, font_size as u16, 1.0).width
}

// --- Modal shell (“chrome”) -------------------------------------------------

/// Props for a standard modal / dialog panel.
#[derive(Clone, Debug)]
pub struct ModalChromeProps<'a> {
    pub rect: Rect,
    pub title: Option<&'a str>,
    pub palette: UiPalette,
    /// Whether this surface is the active keyboard focus target (shows ring).
    pub focused: bool,
    /// Stable id for debugging / future platform hooks (e.g. narration).
    pub semantic_id: &'a str,
}

pub fn draw_modal_chrome(props: &ModalChromeProps<'_>) {
    let _ = props.semantic_id; // reserved for logging / OS integration
    draw_panel(
        props.rect,
        PanelStyle {
            bg: props.palette.panel_bg,
            border: Some((2.0, props.palette.panel_border)),
        },
    );
    if props.focused {
        draw_focus_ring(props.rect, props.palette.focus_ring, 3.0);
    }
    if let Some(title) = props.title {
        draw_text(
            title,
            props.rect.x + 20.0,
            props.rect.y + 44.0,
            32.0,
            props.palette.text_primary,
        );
    }
}

pub fn draw_focus_ring(r: Rect, color: Color, pad: f32) {
    draw_rectangle_lines(
        r.x - pad,
        r.y - pad,
        r.w + pad * 2.0,
        r.h + pad * 2.0,
        2.0,
        color,
    );
}

// --- Loading / async edge cases ---------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum LoadingState<'a> {
    Idle,
    Loading { label: &'a str },
    Error { message: &'a str },
}

/// Draw a non-blocking loading or error affordance inside `inside` (usually modal content area).
pub fn draw_loading_state(
    state: LoadingState<'_>,
    inside: Rect,
    ty: &TypeScale,
    prefs: &UiPreferences,
    elapsed_secs: f32,
) {
    match state {
        LoadingState::Idle => {}
        LoadingState::Loading { label } => {
            let mut y = inside.y + 24.0;
            draw_text(label, inside.x + 20.0, y, ty.body, Color::from_rgba(200, 220, 255, 255));
            y += line_height(ty.body) + 8.0;
            if prefs.reduced_motion {
                draw_text("…", inside.x + 20.0, y, ty.title, Color::from_rgba(180, 200, 240, 255));
            } else {
                let phase = (elapsed_secs * 2.5).sin() * 0.5 + 0.5;
                let dots = match (phase * 3.0).floor() as i32 {
                    0 => ".",
                    1 => "..",
                    _ => "...",
                };
                draw_text(dots, inside.x + 20.0, y, ty.title, Color::from_rgba(180, 200, 240, 255));
            }
        }
        LoadingState::Error { message } => {
            draw_text(
                "Something went wrong",
                inside.x + 20.0,
                inside.y + 24.0,
                ty.title,
                Color::from_rgba(255, 180, 160, 255),
            );
            draw_wrapped_text(
                message,
                inside.x + 20.0,
                inside.y + 24.0 + line_height(ty.title) + 6.0,
                ty.body.max(ty.body_min),
                inside.w - 40.0,
                Color::from_rgba(230, 210, 210, 255),
            );
        }
    }
}

// --- Keyboard affordances ---------------------------------------------------

/// One row: key chord + description (visible copy for keyboard-first play).
#[derive(Clone, Debug)]
pub struct KeyHint<'a> {
    pub keys: &'a str,
    pub description: &'a str,
}

/// Returns the Y coordinate just below the last hint row (for stacking more content).
pub fn draw_key_hints(hints: &[KeyHint<'_>], x: f32, mut y: f32, font: f32, key_color: Color, desc_color: Color) -> f32 {
    let lh = line_height(font);
    for h in hints {
        let key_part = format!("{} — ", h.keys);
        draw_text(&key_part, x, y, font, key_color);
        let kw = measure_w(&key_part, font);
        draw_text(h.description, x + kw, y, font, desc_color);
        y += lh + 4.0;
    }
    y
}

// --- Text -------------------------------------------------------------------

/// Word-wrap using measured widths (default font).
pub fn draw_wrapped_text(text: &str, left: f32, mut y: f32, size: f32, max_width: f32, color: Color) {
    let mut cursor_x = left;
    for word in text.split_whitespace() {
        let w = measure_w(word, size);
        let space_w = measure_w(" ", size);
        if cursor_x > left && cursor_x + w > left + max_width {
            cursor_x = left;
            y += line_height(size);
        }
        draw_text(word, cursor_x, y, size, color);
        cursor_x += w + space_w * 0.35;
    }
}

/// Footer area below a modal so stacked UI avoids overlapping short screens.
pub fn content_area_below_modal(modal_rect: Rect, scale: f32) -> Rect {
    let bottom_margin = layout::safe_margins(scale);
    let y0 = modal_rect.y + modal_rect.h + 12.0 * scale;
    let h = (screen_height() - y0 - bottom_margin).max(0.0);
    Rect::new(
        layout::safe_margins(scale),
        y0,
        screen_width() - layout::safe_margins(scale) * 2.0,
        h,
    )
}
