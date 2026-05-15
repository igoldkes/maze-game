//! In-game modal overlays: win celebration, quit confirmations, and F1 password prompt.

use super::super::ui::{draw_panel, PanelStyle};
use macroquad::prelude::*;

/// `selected` is 0..=2: play again, restart level, main menu.
pub fn draw_end_menu_overlay(
    player_name: &str,
    replay_cmp: Option<(&str, Color)>,
    playtime_secs: Option<f32>,
    selected: usize,
) {
    let w = screen_width();
    let h = screen_height();
    let pw = 720.0;
    let ph = 400.0;
    let x = (w - pw) * 0.5;
    let y = (h - ph) * 0.5;
    draw_panel(
        Rect::new(x, y, pw, ph),
        PanelStyle {
            bg: Color::from_rgba(10, 12, 24, 245),
            border: Some((2.0, Color::from_rgba(130, 150, 220, 255))),
        },
    );
    let name = if player_name.trim().is_empty() {
        "Traveler"
    } else {
        player_name.trim()
    };
    let mut yy = y + 24.0;
    draw_text(
        &format!("Congratulations, {name}!"),
        x + 20.0,
        yy,
        32.0,
        Color::from_rgba(180, 235, 190, 255),
    );
    yy += 40.0;
    draw_text(
        "You've made your way out of the maze.",
        x + 20.0,
        yy,
        22.0,
        Color::from_rgba(210, 215, 235, 255),
    );
    yy += 30.0;
    if let Some(secs) = playtime_secs {
        let time_line = if secs >= 60.0 {
            let m = (secs / 60.0).floor() as u32;
            let s = secs - m as f32 * 60.0;
            format!("Play time this stage: {m}m {s:.1}s")
        } else {
            format!("Play time this stage: {secs:.1}s")
        };
        draw_text(&time_line, x + 20.0, yy, 22.0, Color::from_rgba(160, 220, 255, 255));
        yy += 28.0;
    }
    if let Some((cmp, color)) = replay_cmp {
        draw_text(cmp, x + 20.0, yy, 20.0, color);
        yy += 30.0;
    }
    draw_text(
        "What would you like to do next?",
        x + 20.0,
        yy,
        22.0,
        Color::from_rgba(230, 230, 245, 255),
    );
    yy += 36.0;

    let row_h = 36.0;
    let row_pad_x = 18.0;
    let row_bg_w = pw - row_pad_x * 2.0;
    let labels = [
        "Play again (next stage, larger maze)",
        "Restart level (same stage, same maze)",
        "Return to main menu",
    ];
    let row0_y = yy;
    for i in 0..3 {
        let ry = row0_y + i as f32 * row_h;
        if selected == i {
            draw_rectangle(
                x + row_pad_x,
                ry - 15.0,
                row_bg_w,
                row_h,
                Color::from_rgba(88, 94, 118, 235),
            );
        }
        draw_text(
            labels[i],
            x + row_pad_x + 10.0,
            ry + 8.0,
            20.0,
            Color::from_rgba(230, 235, 255, 255),
        );
    }
    yy = row0_y + 3.0 * row_h + 14.0;
    draw_text(
        "↑ ↓ select · Enter confirm · Esc = main menu (saves run if not yet saved)",
        x + 20.0,
        yy,
        16.0,
        Color::from_rgba(160, 170, 200, 255),
    );
    yy += 22.0;
    draw_text(
        "Your run is saved once to local records when you leave or play the next stage.",
        x + 20.0,
        yy,
        16.0,
        Color::from_rgba(160, 170, 200, 255),
    );
    yy += 20.0;
    draw_text(
        "See Previous records on the title screen.",
        x + 20.0,
        yy,
        16.0,
        Color::from_rgba(160, 170, 200, 255),
    );
}

pub fn draw_normal_f1_password_overlay(normal_f1_password_buffer: &str, normal_f1_password_error: bool) {
    let w = screen_width();
    let h = screen_height();
    let pw = 720.0;
    let ph = 170.0;
    let x = (w - pw) * 0.5;
    let y = (h - ph) * 0.5;
    draw_panel(
        Rect::new(x, y, pw, ph),
        PanelStyle {
            bg: Color::from_rgba(12, 14, 30, 245),
            border: Some((2.0, Color::from_rgba(130, 150, 220, 255))),
        },
    );
    draw_text(
        "Developer debug access (F1) — enter password:",
        x + 16.0,
        y + 34.0,
        26.0,
        Color::from_rgba(220, 228, 255, 255),
    );
    let masked = format!("{}_", "*".repeat(normal_f1_password_buffer.len()));
    draw_text(
        &masked,
        x + 16.0,
        y + 82.0,
        34.0,
        Color::from_rgba(150, 230, 180, 255),
    );
    let msg = if normal_f1_password_error {
        "Wrong password. Enter to retry, Esc to cancel."
    } else {
        "Enter to submit, Esc to cancel."
    };
    draw_text(
        msg,
        x + 16.0,
        y + 124.0,
        20.0,
        Color::from_rgba(235, 185, 170, 255),
    );
}

pub fn draw_quit_confirm_overlay() {
    let w = screen_width();
    let h = screen_height();
    draw_rectangle(0.0, 0.0, w, h, Color::from_rgba(0, 0, 0, 160));
    let pw = 640.0;
    let ph = 150.0;
    let x = (w - pw) * 0.5;
    let y = (h - ph) * 0.5;
    draw_panel(
        Rect::new(x, y, pw, ph),
        PanelStyle {
            bg: Color::from_rgba(12, 14, 28, 245),
            border: Some((2.0, Color::from_rgba(130, 150, 220, 255))),
        },
    );
    draw_text(
        "Return to main menu?",
        x + 20.0,
        y + 36.0,
        28.0,
        Color::from_rgba(220, 225, 245, 255),
    );
    draw_text(
        "Y or Enter = Yes   ·   N or Esc = No   ·   R = Replay",
        x + 20.0,
        y + 88.0,
        22.0,
        Color::from_rgba(160, 200, 255, 255),
    );
}

pub fn draw_unsaved_quit_confirm_overlay() {
    let w = screen_width();
    let h = screen_height();
    draw_rectangle(0.0, 0.0, w, h, Color::from_rgba(0, 0, 0, 175));
    let pw = 760.0;
    let ph = 190.0;
    let x = (w - pw) * 0.5;
    let y = (h - ph) * 0.5;
    draw_panel(
        Rect::new(x, y, pw, ph),
        PanelStyle {
            bg: Color::from_rgba(18, 12, 16, 250),
            border: Some((2.0, Color::from_rgba(210, 140, 120, 255))),
        },
    );
    draw_text(
        "Go back to main lobby now?",
        x + 20.0,
        y + 42.0,
        30.0,
        Color::from_rgba(255, 220, 210, 255),
    );
    draw_text(
        "Your current stage progress will NOT be saved if you leave before",
        x + 20.0,
        y + 87.0,
        22.0,
        Color::from_rgba(255, 180, 160, 255),
    );
    draw_text(
        "finishing this maze.",
        x + 20.0,
        y + 113.0,
        22.0,
        Color::from_rgba(255, 180, 160, 255),
    );
    draw_text(
        "Y / Enter = leave anyway    ·    N / Esc = keep playing   ·   R = Replay",
        x + 20.0,
        y + 155.0,
        22.0,
        Color::from_rgba(220, 230, 255, 255),
    );
}
