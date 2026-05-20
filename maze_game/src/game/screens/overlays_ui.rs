//! In-game modal overlays: win celebration, quit confirmations, and F1 password prompt.

use super::super::ui::components::{draw_modal_chrome, draw_wrapped_text, ModalChromeProps};
use super::super::ui::layout::{centered_clamped_rect, safe_margins, scaled_type, ui_scale};
use super::super::ui::theme::{TypeScale, UiPreferences};
use super::super::ui::{draw_panel, PanelStyle};
use macroquad::prelude::*;

use super::super::PauseMenuState;

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

pub fn draw_quit_confirm_overlay( menu_state: PauseMenuState ) {
    match menu_state {
        PauseMenuState::Menu { pause_menu_role } => {
            let prefs = UiPreferences::default();
            let palette = prefs.palette();
            let scale = ui_scale();
            let margin = safe_margins(scale);
            let ty = scaled_type(&TypeScale::default(), scale);

            let w = screen_width();
            let h = screen_height();
            draw_rectangle(0.0, 0.0, w, h, Color::from_rgba(0, 0, 0, 160));
            let pw = 640.0 * scale; // 640.0
            let ph = 285.0 * scale; // 150.0

            let rect = centered_clamped_rect(pw, ph, margin);
            let x = rect.x;
            let y = rect.y;

            let row0_y = y + 92.0 * scale;
            let row_h = 38.0 * scale;
            let row_pad_x = 18.0 * scale;
            let row_bg_w = rect.w - row_pad_x * 2.0;

            draw_panel(
                Rect::new(x, y, pw, ph),
                PanelStyle {
                    bg: Color::from_rgba(12, 14, 28, 245),
                    border: Some((2.0, Color::from_rgba(130, 150, 220, 255))),
                },
            );
            draw_text(
                "Game Paused",
                x + 20.0,
                y + 40.0,
                40.0,
                Color::from_rgba(220, 225, 245, 255),
            );

            let labels: [&str; 4] = [
                "Resume Game",
                "Settings",
                "Return to Main Menu",
                "Exit to Desktop",
            ];

            for i in 0..4 {
                let ry = row0_y + (1.25 * (i as f32)) * row_h;
                if pause_menu_role == i {
                    draw_rectangle(
                        x + row_pad_x,
                        ry - 15.0 * scale,
                        row_bg_w,
                        row_h,
                        Color::from_rgba(88, 94, 118, 235),
                    );
                }
                let label = labels[i];
                draw_text(
                    label,
                    x + row_pad_x + 10.0 * scale,
                    ry + 8.0 * scale,
                    ty.body + 4.0,
                    palette.text_primary,
                );
            }
        }
        PauseMenuState::Settings { pause_settings_menu_role, menu_music_settings_toggle, maze_music_settings_toggle, footstep_settings_toggle, wind_rain_settings_toggle, menu_clicks_settings_toggle } => {
            let prefs = UiPreferences::default();
            let palette = prefs.palette();
            let scale = ui_scale();
            let margin = safe_margins(scale);
            let ty = scaled_type(&TypeScale::default(), scale);

            let w = screen_width();
            let h = screen_height();
            draw_rectangle(0.0, 0.0, w, h, Color::from_rgba(0, 0, 0, 160));
            let pw = 760.0 * scale;
            let ph = 300.0 * scale;

            let rect = centered_clamped_rect(pw, ph, margin);
            let x = rect.x;
            let y = rect.y;

            let row0_y = y + 92.0 * scale;
            let row_h = 38.0 * scale;
            let row_pad_x = 18.0 * scale;
            let row_bg_w = rect.w - row_pad_x * 2.0;

            let semantic_id = "in_game_settings";

            draw_modal_chrome(&ModalChromeProps {
                rect,
                title: None,
                palette,
                focused: true,
                semantic_id,
            });

            draw_text(
                "Settings",
                x + 20.0,
                y + 44.0 * scale,
                ty.headline,
                palette.text_primary,
            );

            let labels: [&str; 5] = [
                    "Menu Music",
                    "Maze Music",
                    "Footsteps",
                    "Wind and Rain",
                    "Menu Clicks",
            ];
            for i in 0..5 {
                let ry = row0_y + i as f32 * row_h;
                if pause_settings_menu_role == i {
                    draw_rectangle(
                        x + row_pad_x,
                        ry - 15.0 * scale,
                        row_bg_w,
                        row_h,
                        Color::from_rgba(88, 94, 118, 235),
                    );
                }
                let label = labels[i];
                draw_text(
                    label,
                    x + row_pad_x + 10.0 * scale,
                    ry + 8.0 * scale,
                    ty.body,
                    palette.text_primary,
                );
            }

            let row0_y_opt = y + 92.0 * scale;
            let labels: [&str; 5] = [
                    if menu_music_settings_toggle { "On" } else { "Off" },
                    if maze_music_settings_toggle { "On" } else { "Off" },
                    if footstep_settings_toggle { "On" } else { "Off" },
                    if wind_rain_settings_toggle { "On" } else { "Off" },
                    if menu_clicks_settings_toggle { "On" } else { "Off" },
            ];
            for i in 0..5 {
                let ry = row0_y_opt + i as f32 * row_h;
                
                let label = labels[i];

                if label == "On" {
                    draw_rectangle(
                        x + row_pad_x + 150.0 * scale,
                        ry - 11.0 * scale,
                        40.0 * scale,
                        row_h - 8.0,
                        Color::from_rgba(88, 94, 150, 235),
                    );
                    draw_text(
                        label,
                        x + row_pad_x + 160.0 * scale,
                        ry + 8.0 * scale,
                        ty.body,
                        palette.text_primary,
                        //Color::from_rgba(10, 163, 13, 1),
                    );
                } else {
                    draw_rectangle(
                        x + row_pad_x + 150.0 * scale,
                        ry - 11.0 * scale,
                        40.0 * scale,
                        row_h - 8.0,
                        Color::from_rgba(88, 94, 150, 235),
                    );
                    draw_text(
                        label,
                        x + row_pad_x + 157.5 * scale,
                        ry + 8.0 * scale,
                        ty.body,
                        palette.text_primary,
                        //Color::from_rgba(163, 10, 10, 1),
                    );
                }
            }
        }
        PauseMenuState::None => {}
    }
    
}

pub fn draw_in_game_settings_overlay( pause_settings_menu_role: usize ) {
    let prefs = UiPreferences::default();
    let palette = prefs.palette();
    let scale = ui_scale();
    let margin = safe_margins(scale);
    let ty = scaled_type(&TypeScale::default(), scale);

    let w = screen_width();
    let h = screen_height();
    draw_rectangle(0.0, 0.0, w, h, Color::from_rgba(0, 0, 0, 160));
    let pw = 640.0 * scale; // 640.0
    let ph = 285.0 * scale; // 150.0

    let rect = centered_clamped_rect(pw, ph, margin);
    let x = rect.x;
    let y = rect.y;

    let row0_y = y + 92.0 * scale;
    let row_h = 38.0 * scale;
    let row_pad_x = 18.0 * scale;
    let row_bg_w = rect.w - row_pad_x * 2.0;

    let semantic_id = "in_game_settings";

    draw_modal_chrome(&ModalChromeProps {
        rect,
        title: None,
        palette,
        focused: true,
        semantic_id,
    });
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
        "Go back to main menu now?",
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
        "Y / Enter = leave anyway    ·    N / Esc = keep playing",
        x + 20.0,
        y + 155.0,
        22.0,
        Color::from_rgba(220, 230, 255, 255),
    );
}
