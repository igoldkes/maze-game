//! Intro narrative and tutorial flow.

use macroquad::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

pub const OPENING_MAP_SECS: f32 = 10.0;
pub const NAME_MAX_LEN: usize = 32;
static NAME_INPUT_QUEUE_CLEARED: AtomicBool = AtomicBool::new(false);

/// Whether queued key events were drained after entering the in-run name-on-map step.
pub fn name_on_map_input_ready() -> bool {
    NAME_INPUT_QUEUE_CLEARED.load(Ordering::Relaxed)
}

#[derive(Clone, Debug)]
pub enum StoryPhase {
    IntroThought,
    IntroExplainAskMap { is_first_stage: bool },
    MapReveal {
        elapsed: f32,
        is_first_stage: bool,
    },
    AskNameOnMap {
        buffer: String,
        backspace_cool: f32,
    },
    ControlsPrompt,
    Playing,
    Won,
    Restart,
}

impl StoryPhase {
    pub fn new_run() -> Self {
        StoryPhase::IntroThought
    }

    pub fn update(&mut self, dt: f32) -> Option<String> {
        match self {
            StoryPhase::IntroThought => {
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                    *self = StoryPhase::IntroExplainAskMap { is_first_stage: true };
                }
                None
            }
            StoryPhase::IntroExplainAskMap { is_first_stage } => {
                if is_key_pressed(KeyCode::Space) {
                    *self = StoryPhase::MapReveal { elapsed: 0.0, is_first_stage: *is_first_stage };
                }
                None
            }
            StoryPhase::MapReveal { elapsed, is_first_stage } => {
                *elapsed += dt;
                if *elapsed >= OPENING_MAP_SECS {
                    NAME_INPUT_QUEUE_CLEARED.store(false, Ordering::Relaxed);
                    if *is_first_stage {
                        *self = StoryPhase::AskNameOnMap {
                            buffer: String::new(),
                            backspace_cool: 0.0,
                        };
                    } else {
                        *self = StoryPhase::Playing;
                    }
                    }
                None
            }
            StoryPhase::AskNameOnMap {
                buffer,
                backspace_cool,
            } => {
                *backspace_cool = (*backspace_cool - dt).max(0.0);
                if !NAME_INPUT_QUEUE_CLEARED.load(Ordering::Relaxed) {
                    // Drop any queued characters from previous phases before accepting a new name.
                    for _ in 0..4096 {
                        if get_char_pressed().is_none() {
                            break;
                        }
                    }
                    NAME_INPUT_QUEUE_CLEARED.store(true, Ordering::Relaxed);
                    return None;
                }

                while let Some(ch) = get_char_pressed() {
                    // Some platforms deliver Backspace/Delete via character events; KeyCode::Backspace
                    // alone would miss those.
                    if ch == '\u{8}' || ch == '\u{7f}' {
                        if !buffer.is_empty() {
                            buffer.pop();
                            *backspace_cool = 0.12;
                        }
                        continue;
                    }
                    if ch.is_control() {
                        continue;
                    }
                    if ch.is_ascii_alphanumeric() && buffer.len() < NAME_MAX_LEN {
                        buffer.push(ch);
                    }
                }

                if is_key_pressed(KeyCode::Enter) {
                    let name = buffer.trim().to_string();
                    if name.is_empty() {
                        return None;
                    }
                    *self = StoryPhase::ControlsPrompt;
                    return Some(name);
                }

                let back_press =
                    is_key_pressed(KeyCode::Backspace) || is_key_pressed(KeyCode::Delete);
                let back_held = is_key_down(KeyCode::Backspace) || is_key_down(KeyCode::Delete);
                if !buffer.is_empty() && *backspace_cool <= 0.0 {
                    if back_press {
                        buffer.pop();
                        *backspace_cool = 0.15;
                    } else if back_held {
                        buffer.pop();
                        *backspace_cool = 0.07;
                    }
                }
                None
            }
            StoryPhase::ControlsPrompt => {
                if is_key_pressed(KeyCode::Enter) {
                    *self = StoryPhase::Playing;
                }
                None
            }
            StoryPhase::Restart => {
                *self = StoryPhase::Playing;
                None
            },
            StoryPhase::Playing | StoryPhase::Won => None,
        }
    }

    /// Maze map should be visible on this phase.
    pub fn show_map_item(&self) -> bool {
        matches!(self, StoryPhase::MapReveal { .. })
    }

    /// Parchment + black backdrop intro (no live world underlay).
    pub fn show_paper_overlay(&self) -> bool {
        matches!(
            self,
            StoryPhase::MapReveal { .. } | StoryPhase::AskNameOnMap { .. }
        )
    }

    pub fn movement_allowed(&self) -> bool {
        matches!(self, StoryPhase::Playing)
    }

    pub fn set_won(&mut self) {
        *self = StoryPhase::Won;
    }
}

pub fn opening_story_lines() -> &'static [&'static str] {
    &["Where am I?"]
}

pub fn draw_story_prompt(line1: &str, line2: &str, hint: &str) {
    let w = screen_width();
    let h = screen_height();
    let panel_w = 760.0;
    let panel_h = 132.0;
    let px = (w - panel_w) / 2.0;
    let py = h * 0.70;

    draw_rectangle(px, py, panel_w, panel_h, Color::from_rgba(10, 12, 22, 230));
    draw_rectangle_lines(px, py, panel_w, panel_h, 2.0, Color::from_rgba(120, 140, 200, 255));

    draw_text(
        line1,
        px + 16.0,
        py + 28.0,
        20.0,
        Color::from_rgba(200, 210, 240, 255),
    );
    draw_text(line2, px + 16.0, py + 58.0, 20.0, Color::from_rgba(200, 210, 240, 255));
    draw_text(
        hint,
        px + 16.0,
        py + 98.0,
        20.0,
        Color::from_rgba(140, 220, 180, 255),
    );
}

pub fn draw_name_prompt(buffer: &str, input_ready: bool) {
    let w = screen_width();
    let h = screen_height();
    let panel_w = 700.0;
    let panel_h = 160.0;
    let px = (w - panel_w) / 2.0;
    let py = h * 0.69;
    draw_rectangle(px, py, panel_w, panel_h, Color::from_rgba(10, 12, 22, 255));
    draw_rectangle_lines(px, py, panel_w, panel_h, 2.0, Color::from_rgba(120, 140, 200, 255));
    draw_text(
        "Write your name on the map, then press Enter.",
        px + 16.0,
        py + 30.0,
        22.0,
        Color::from_rgba(210, 220, 250, 255),
    );
    let line = if !input_ready {
        "_"
    } else if buffer.is_empty() {
        "_"
    } else {
        buffer
    };
    draw_text(line, px + 16.0, py + 78.0, 30.0, Color::from_rgba(140, 220, 180, 255));
}

pub fn draw_phase_banner(phase: &StoryPhase, _player_name: &str) {
    let bx = 8.0;
    let by = screen_height() - 52.0;
    let bw = 680.0;
    let bh = 30.0;
    draw_rectangle(bx, by, bw, bh, Color::from_rgba(8, 10, 20, 185));
    match phase {
        StoryPhase::MapReveal { elapsed, is_first_stage: _ } => {
            let left = (OPENING_MAP_SECS - elapsed).max(0.0);
            draw_text(
                &format!("Map fades in {:.0}s", left.ceil()),
                12.0,
                screen_height() - 36.0,
                28.0,
                Color::from_rgba(255, 232, 120, 255),
            );
        }
        StoryPhase::IntroThought => {}
        StoryPhase::IntroExplainAskMap { is_first_stage: _ } => {}
        StoryPhase::AskNameOnMap { .. } => {}
        StoryPhase::ControlsPrompt => {}
        StoryPhase::Playing => {}
        StoryPhase::Restart => {}
        StoryPhase::Won => {
            draw_text(
                "You found the exit.",
                12.0,
                screen_height() - 36.0,
                24.0,
                Color::from_rgba(120, 255, 180, 255),
            );
        }
    }
}
