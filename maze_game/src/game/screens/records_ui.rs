//! Startup records panel UI rendering.

use super::super::progress::ProgressService;
use macroquad::prelude::*;

pub fn draw_startup_records_panel(
    progress: &ProgressService,
    startup_records_scroll: usize,
    startup_records_selected: usize,
) {
    let w = screen_width();
    let h = screen_height();
    let pw = (w * 0.92).min(1040.0);
    let ph = (h * 0.68).min(480.0);
    let x = (w - pw) * 0.5;
    let y = (h - ph) * 0.5 - 20.0;
    draw_rectangle(x, y, pw, ph, Color::from_rgba(10, 12, 24, 248));
    draw_rectangle_lines(x, y, pw, ph, 2.0, Color::from_rgba(130, 150, 220, 255));

    draw_text(
        "Previous records (this PC)",
        x + 18.0,
        y + 30.0,
        28.0,
        Color::from_rgba(220, 230, 255, 255),
    );
    let path_hint = format!("Saved under: {}", progress.path().display());
    draw_text(
        &path_hint,
        x + 18.0,
        y + 53.0,
        17.0,
        Color::from_rgba(150, 170, 210, 255),
    );

    let leaderboard = progress.leaderboard_max_stage_per_player();
    let split_x = x + pw * 0.56;
    let right_x = split_x + 8.0;

    // --- Leaderboard column (max stage per nickname) ---
    draw_text(
        "Leaderboard (highest stage)",
        right_x,
        y + 78.0,
        20.0,
        Color::from_rgba(200, 220, 255, 255),
    );
    let mut ly = y + 108.0;
    if leaderboard.is_empty() {
        draw_text(
            "No entries yet.",
            right_x,
            ly,
            17.0,
            Color::from_rgba(170, 180, 210, 255),
        );
    } else {
        const MAX_LB: usize = 14;
        for (rank, (name, stage)) in leaderboard.iter().take(MAX_LB).enumerate() {
            let line = format!("{}. {} — stage {}", rank + 1, name, stage);
            draw_text(&line, right_x, ly, 17.0, Color::from_rgba(210, 215, 240, 255));
            ly += 22.0;
        }
        if leaderboard.len() > MAX_LB {
            draw_text(
                "…",
                right_x,
                ly,
                17.0,
                Color::from_rgba(150, 160, 190, 255),
            );
        }
    }

    // --- Newest-first log (left) ---
    let recs = progress.load_summaries_newest_first(50);
    const PAGE: usize = 14;
    let max_scroll = recs.len().saturating_sub(PAGE);
    let scroll = startup_records_scroll.min(max_scroll);

    let mut yy = y + 78.0;
    if recs.is_empty() {
        let hint = if progress.has_saved_records() {
            "Save file is not empty but no line matched the current format (old saves may need migration)."
        } else {
            "No readable saves yet — win a maze and choose to save once."
        };
        draw_text(hint, x + 18.0, yy, 15.0, Color::from_rgba(200, 200, 220, 255));
        return;
    }

    draw_text(
        "Recent runs (newest first)",
        x + 18.0,
        yy,
        20.0,
        Color::from_rgba(160, 200, 255, 255),
    );
    yy += 22.0;
    draw_text(
        "player / stage / size / gen / time / …",
        x + 18.0,
        yy,
        17.0,
        Color::from_rgba(140, 170, 210, 255),
    );
    yy += 22.0;
    for (idx, r) in recs.iter().enumerate().skip(scroll).take(PAGE) {
        let run_kind = if r.is_replay { "[R]" } else { "[N]" };
        let line = format!(
            "{} · S{} · {}×{} · {} · {:.1}s · st({},{}) ex({},{}) · {}",
            r.player_name,
            r.stage,
            r.maze_w,
            r.maze_h,
            r.maze_generator,
            r.elapsed_secs,
            r.start_x,
            r.start_y,
            r.exit_x,
            r.exit_y,
            run_kind
        );
        let color = if idx == startup_records_selected {
            Color::from_rgba(245, 245, 255, 255)
        } else {
            Color::from_rgba(210, 215, 235, 255)
        };
        let prefix = if idx == startup_records_selected { ">" } else { " " };
        draw_text(&format!("{prefix} {line}"), x + 18.0, yy, 17.0, color);
        yy += 22.0;
    }
    if recs.len() > PAGE {
        draw_text(
            "Up / Down — select   ·   Enter — replay   ·   Esc — back",
            x + 18.0,
            y + ph - 32.0,
            16.0,
            Color::from_rgba(140, 220, 180, 255),
        );
    } else {
        draw_text(
            "Enter — replay selected   ·   Esc — back",
            x + 18.0,
            y + ph - 32.0,
            16.0,
            Color::from_rgba(140, 220, 180, 255),
        );
    }
}
