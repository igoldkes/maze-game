//! Startup screen UI rendering (role prompt, dev password, records entry, and footer credits).

use super::records_ui::draw_startup_records_panel;
use super::super::progress::ProgressService;
use super::super::story::NAME_MAX_LEN;
use super::super::ui::components::{draw_modal_chrome, draw_wrapped_text, ModalChromeProps};
use super::super::ui::layout::{centered_clamped_rect, safe_margins, scaled_type, ui_scale};
use super::super::ui::theme::{TypeScale, UiPreferences};
use super::super::StartupState;
use macroquad::prelude::*;

#[allow(clippy::too_many_arguments)]
pub fn draw_startup_overlay(
    startup: &StartupState,
    progress: &ProgressService,
    password_buffer: &str,
    startup_records_scroll: usize,
    startup_records_selected: usize,
    player_name: &str,
    startup_nickname_buffer: &str,
    nickname_input_queue_ready: bool,
    continue_next_stage: Option<usize>,
    continue_max_cleared: Option<usize>,
    menu_role: usize,
    menu_run_type: usize,
    menu_continue: usize,
    back_confirm: bool,
    menu_music_settings_toggle: bool,
    maze_music_settings_toggle: bool,
    footstep_settings_toggle: bool,
    wind_rain_settings_toggle: bool,
    menu_clicks_settings_toggle: bool,
) {
    let w = screen_width();
    let h = screen_height();
    draw_rectangle(0.0, 0.0, w, h, Color::from_rgba(0, 0, 0, 180));

    if *startup == StartupState::ViewRecords {
        draw_startup_records_panel(progress, startup_records_scroll, startup_records_selected);
        draw_startup_credits_footer();
        if back_confirm {
            draw_startup_back_confirm_layer();
        }
        return;
    }

    let prefs = UiPreferences::default();
    let palette = prefs.palette();
    let scale = ui_scale();
    let margin = safe_margins(scale);
    let ty = scaled_type(&TypeScale::default(), scale);

    let preferred_h = match startup {
        StartupState::Splash => 240.0,
        StartupState::AskNewOrContinue => 280.0,
        StartupState::AskPlayerRole => {
            if progress.has_saved_records() {
                340.0
            } else {
                300.0
            }
        }
        StartupState::AskPlayerName
        | StartupState::AskContinueFromLog
        | StartupState::NicknameMustChangeNotice
        | StartupState::ContinueNoRecordNotice => 300.0,
        StartupState::Settings => 300.0,
        _ => 220.0,
    };
    let pw = 760.0 * scale;
    let ph = preferred_h * scale;
    let rect = centered_clamped_rect(pw, ph, margin);
    let x = rect.x;
    let y = rect.y;

    let semantic_id = match startup {
        StartupState::Splash => "startup-splash",
        StartupState::AskPlayerRole => "startup-ask-role",
        StartupState::AskNewOrContinue => "startup-new-or-continue",
        StartupState::AskPlayerName => "startup-nickname",
        StartupState::AskContinueFromLog => "startup-continue-prompt",
        StartupState::ContinueNoRecordNotice => "startup-no-record-for-name",
        StartupState::NicknameMustChangeNotice => "startup-nickname-blocked",
        StartupState::AskDevPassword => "startup-dev-password",
        StartupState::ViewRecords | StartupState::Done => "startup-unused",
        StartupState::Settings => "startup-settings",
    };
    draw_modal_chrome(&ModalChromeProps {
        rect,
        title: None,
        palette,
        focused: true,
        semantic_id,
    });

    let row_h = 38.0 * scale;
    let row_pad_x = 18.0 * scale;
    let row_bg_w = rect.w - row_pad_x * 2.0;

    match startup {
        StartupState::Splash => {
            draw_text(
                "Maze Game",
                x + 20.0,
                y + 72.0 * scale,
                ty.headline + 8.0 * scale,
                palette.text_primary,
            );
            draw_text(
                "Press any key to start",
                x + 20.0,
                y + 130.0 * scale,
                ty.title,
                palette.accent_ok,
            );
            draw_text(
                "Keyboard controls are shown before each step.",
                x + 20.0,
                y + 170.0 * scale,
                ty.body_min,
                palette.text_muted,
            );
        }
        StartupState::AskPlayerRole => {
            draw_text(
                "Are you a player?",
                x + 20.0,
                y + 44.0 * scale,
                ty.headline,
                palette.text_primary,
            );
            let row0_y = y + 92.0 * scale;
            let n = if progress.has_saved_records() { 5 } else { 4 };

            if n == 5 {
                let labels: [&str; 5] = [
                    "Player Mode",
                    "Developer / Test Mode (password)",
                    "Previous Records (cleared mazes on this computer)",
                    "Settings",
                    "Exit Game",
                ];
                for i in 0..n {
                    let ry = row0_y + i as f32 * row_h;
                    if menu_role == i {
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
            } else {
                    let labels: [&str; 4] = [
                    "Player Mode",
                    "Developer / Test Mode (password)",
                    "Settings",
                    "Exit Game",
                ];
                for i in 0..n {
                    let ry = row0_y + i as f32 * row_h;
                    if menu_role == i {
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
            }
            
            draw_text(
                "↑ ↓ select · Enter confirm · Esc back",
                x + 20.0,
                row0_y + n as f32 * row_h + 14.0 * scale,
                ty.body_min,
                palette.text_muted,
            );
        }
        StartupState::AskNewOrContinue => {
            draw_text(
                "Start Type",
                x + 20.0,
                y + 44.0 * scale,
                ty.headline,
                palette.text_primary,
            );
            let row0_y = y + 92.0 * scale;
            let labels = [
                "New game (fresh intro story)",
                "Continue (ask nickname and resume)",
            ];
            for i in 0..2 {
                let ry = row0_y + i as f32 * row_h;
                if menu_run_type == i {
                    draw_rectangle(
                        x + row_pad_x,
                        ry - 15.0 * scale,
                        row_bg_w,
                        row_h,
                        Color::from_rgba(88, 94, 118, 235),
                    );
                }
                draw_text(
                    labels[i],
                    x + row_pad_x + 10.0 * scale,
                    ry + 8.0 * scale,
                    ty.body,
                    palette.text_primary,
                );
            }
            draw_text(
                "↑ ↓ select · Enter confirm · Esc back",
                x + 20.0,
                row0_y + 2.0 * row_h + 14.0 * scale,
                ty.body_min,
                palette.text_muted,
            );
        }
        StartupState::AskPlayerName => {
            draw_text(
                "Enter your nickname to continue",
                x + 20.0,
                y + 44.0 * scale,
                ty.headline,
                palette.text_primary,
            );
            draw_text(
                &format!(
                    "Letters and numbers only, max {} characters. Esc = back.",
                    NAME_MAX_LEN
                ),
                x + 20.0,
                y + 86.0 * scale,
                ty.body_min,
                palette.text_secondary,
            );
            let line = if !nickname_input_queue_ready {
                "_"
            } else if startup_nickname_buffer.is_empty() {
                "_"
            } else {
                startup_nickname_buffer
            };
            draw_text(
                line,
                x + 20.0,
                y + 128.0 * scale,
                ty.title,
                palette.accent_ok,
            );
            draw_text(
                "Press Enter to continue",
                x + 20.0,
                y + 178.0 * scale,
                ty.body,
                palette.accent_ok,
            );
        }
        StartupState::AskContinueFromLog => {
            let max_c = continue_max_cleared.unwrap_or(0);
            let next_s = continue_next_stage.unwrap_or(1);
            draw_text(
                "Continue previous progress?",
                x + 20.0,
                y + 40.0 * scale,
                ty.headline,
                palette.text_primary,
            );
            draw_text(
                &format!(
                    "This computer has saves for \"{}\" (cleared through stage {}).",
                    player_name, max_c
                ),
                x + 20.0,
                y + 86.0 * scale,
                ty.body,
                palette.text_secondary,
            );
            let row0_y = y + 118.0 * scale;
            let yes_line = format!("Yes — continue from stage {}", next_s);
            let labels = [yes_line.as_str(), "No — use a different nickname"];
            for i in 0..2 {
                let ry = row0_y + i as f32 * row_h;
                if menu_continue == i {
                    draw_rectangle(
                        x + row_pad_x,
                        ry - 15.0 * scale,
                        row_bg_w,
                        row_h,
                        Color::from_rgba(88, 94, 118, 235),
                    );
                }
                draw_text(
                    labels[i],
                    x + row_pad_x + 10.0 * scale,
                    ry + 8.0 * scale,
                    ty.body,
                    palette.text_primary,
                );
            }
            draw_text(
                "↑ ↓ select · Enter confirm · Esc back",
                x + 20.0,
                row0_y + 2.0 * row_h + 10.0 * scale,
                ty.body_min,
                palette.text_muted,
            );
        }
        StartupState::NicknameMustChangeNotice => {
            draw_text(
                "Different nickname required",
                x + 20.0,
                y + 40.0 * scale,
                ty.headline,
                palette.text_primary,
            );
            let body = format!(
                "You chose not to continue the saved progress for \"{}\". To start a brand-new run, \
                 you must use a different nickname (letters and numbers only). Press Enter to go back \
                 and enter another name.",
                player_name
            );
            let wrap_w = rect.w - 40.0;
            draw_wrapped_text(
                &body,
                x + 20.0,
                y + 84.0 * scale,
                ty.body_min,
                wrap_w,
                palette.text_secondary,
            );
            draw_text(
                "Esc — back to continue prompt",
                x + 20.0,
                y + 220.0 * scale,
                ty.body_min,
                palette.text_muted,
            );
        }
        StartupState::AskDevPassword => {
            draw_text(
                "Developer test mode password:",
                x + 20.0,
                y + 52.0 * scale,
                ty.headline,
                palette.text_primary,
            );
            let masked = "*".repeat(password_buffer.len());
            draw_text(
                &format!("{}_", masked),
                x + 20.0,
                y + 102.0 * scale,
                ty.title,
                palette.accent_ok,
            );
            draw_text(
                "Press Enter to submit. Wrong password: choose a player nickname next.",
                x + 20.0,
                y + 146.0 * scale,
                ty.body_min,
                palette.text_secondary,
            );
            if progress.has_saved_records() {
                draw_text(
                    "Open “Previous records” from the player role menu.",
                    x + 20.0,
                    y + 178.0 * scale,
                    ty.body_min,
                    palette.text_muted,
                );
            }
        }
        StartupState::ContinueNoRecordNotice => {
            draw_text(
                "No saved record found",
                x + 20.0,
                y + 40.0 * scale,
                ty.headline,
                palette.text_primary,
            );
            let body = format!(
                "No local cleared-stage log was found for \"{}\". Press Enter to try another nickname, or Esc to go back and choose New game.",
                player_name
            );
            let wrap_w = rect.w - 40.0;
            draw_wrapped_text(
                &body,
                x + 20.0,
                y + 84.0 * scale,
                ty.body_min,
                wrap_w,
                palette.text_secondary,
            );
        }
        StartupState::Settings => {
            println!("{}", preferred_h);
            draw_text(
                "Settings",
                x + 20.0,
                y + 44.0 * scale,
                ty.headline,
                palette.text_primary,
            );
            let row0_y = y + 92.0 * scale;
            let labels: [&str; 5] = [
                    "Menu Music",
                    "Maze Music",
                    "Footsteps",
                    "Wind and Rain",
                    "Menu Clicks",
            ];
            for i in 0..5 {
                let ry = row0_y + i as f32 * row_h;
                if menu_role == i {
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
        StartupState::ViewRecords | StartupState::Done => {}
        
    }
    draw_startup_credits_footer();

    if back_confirm {
        draw_startup_back_confirm_layer();
    }
}

fn draw_startup_back_confirm_layer() {
    let w = screen_width();
    let h = screen_height();
    draw_rectangle(0.0, 0.0, w, h, Color::from_rgba(0, 0, 0, 200));
    let pw = 520.0;
    let ph = 160.0;
    let px = (w - pw) * 0.5;
    let py = (h - ph) * 0.5;
    draw_rectangle(px, py, pw, ph, Color::from_rgba(14, 16, 28, 250));
    draw_rectangle_lines(px, py, pw, ph, 2.0, Color::from_rgba(130, 150, 220, 255));
    draw_text(
        "Go back?",
        px + 24.0,
        py + 36.0,
        26.0,
        Color::from_rgba(220, 225, 245, 255),
    );
    draw_text(
        "You will leave this step.",
        px + 24.0,
        py + 72.0,
        18.0,
        Color::from_rgba(180, 190, 220, 255),
    );
    draw_text(
        "Y / Enter — yes    N / Esc — stay",
        px + 24.0,
        py + 108.0,
        18.0,
        Color::from_rgba(140, 210, 255, 255),
    );
}

fn draw_startup_credits_footer() {
    let scale = ui_scale();
    let pad = safe_margins(scale);
    let mut yy = screen_height() - 118.0 * scale;
    let dim = Color::from_rgba(150, 165, 200, 220);
    draw_text(
        "Credits — assets:",
        pad,
        yy,
        17.0,
        Color::from_rgba(200, 210, 235, 255),
    );
    yy += 22.0;
    draw_text(
        "mazeexitstar.png · soulofkiran.itch.io/pixel-art-animated-star (Narik)",
        pad,
        yy,
        15.0,
        dim,
    );
    yy += 20.0;
    draw_text(
        "hintitem.png · twiceuponatime.itch.io/waving-triangular-flag (TwiceUponATime)",
        pad,
        yy,
        15.0,
        dim,
    );
    yy += 20.0;
    draw_text(
        "mapimage.png · opengameart.org/content/old-parchment-paper (cron)",
        pad,
        yy,
        15.0,
        dim,
    );
}
