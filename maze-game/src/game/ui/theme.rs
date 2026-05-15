#![allow(dead_code)]
// Tokens are consumed gradually as more screens use the design system.

//! Design tokens: spacing, typography floors, and contrast-aware palettes.
//!
//! Macroquad has no DOM or screen-reader bridge; “accessibility” here means:
//! keyboard-first flows, readable type scales, focus visibility, and optional high contrast.

use macroquad::prelude::*;

/// Logical reference height (matches default window in `main.rs`).
pub const REFERENCE_SCREEN_H: f32 = 720.0;

#[derive(Clone, Copy, Debug)]
pub struct Spacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

impl Default for Spacing {
    fn default() -> Self {
        Self {
            xs: 6.0,
            sm: 12.0,
            md: 18.0,
            lg: 24.0,
            xl: 36.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TypeScale {
    /// Never draw body copy smaller than this (readability floor).
    pub body_min: f32,
    pub body: f32,
    pub title: f32,
    pub headline: f32,
}

impl Default for TypeScale {
    fn default() -> Self {
        Self {
            body_min: 16.0,
            body: 20.0,
            title: 26.0,
            headline: 34.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UiPalette {
    pub panel_bg: Color,
    pub panel_border: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub accent_ok: Color,
    pub accent_warn: Color,
    pub focus_ring: Color,
}

impl UiPalette {
    pub fn standard() -> Self {
        Self {
            panel_bg: Color::from_rgba(10, 12, 24, 245),
            panel_border: Color::from_rgba(130, 150, 220, 255),
            text_primary: WHITE,
            text_secondary: Color::from_rgba(200, 210, 240, 255),
            text_muted: Color::from_rgba(160, 170, 200, 255),
            accent_ok: Color::from_rgba(140, 220, 180, 255),
            accent_warn: Color::from_rgba(255, 210, 160, 255),
            focus_ring: Color::from_rgba(255, 240, 120, 255),
        }
    }

    /// Higher luminance borders/text for low-vision players (toggle via [`UiPreferences`]).
    pub fn high_contrast() -> Self {
        Self {
            panel_bg: Color::from_rgba(0, 0, 0, 250),
            panel_border: Color::from_rgba(255, 255, 100, 255),
            text_primary: Color::from_rgba(255, 255, 255, 255),
            text_secondary: Color::from_rgba(255, 255, 220, 255),
            text_muted: Color::from_rgba(220, 220, 220, 255),
            accent_ok: Color::from_rgba(120, 255, 180, 255),
            accent_warn: Color::from_rgba(255, 180, 80, 255),
            focus_ring: Color::from_rgba(255, 255, 0, 255),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UiPreferences {
    pub high_contrast: bool,
    /// Skip pulsing/spinner animations (motion sensitivity).
    pub reduced_motion: bool,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            high_contrast: false,
            reduced_motion: false,
        }
    }
}

impl UiPreferences {
    pub fn palette(&self) -> UiPalette {
        if self.high_contrast {
            UiPalette::high_contrast()
        } else {
            UiPalette::standard()
        }
    }
}
