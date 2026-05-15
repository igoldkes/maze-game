//! In-game credits overlay UI.

use macroquad::prelude::*;

pub fn draw_credits_overlay(show_credits: bool) {
    if !show_credits {
        return;
    }
    let w = screen_width();
    let h = screen_height();
    let pw = (w * 0.82).min(920.0);
    let ph = (h * 0.72).min(520.0);
    let x = (w - pw) * 0.5;
    let y = (h - ph) * 0.5;
    draw_rectangle(0.0, 0.0, w, h, Color::from_rgba(0, 0, 0, 180));
    draw_rectangle(x, y, pw, ph, Color::from_rgba(10, 12, 24, 245));
    draw_rectangle_lines(x, y, pw, ph, 2.0, Color::from_rgba(130, 150, 220, 255));

    let mut yy = y + 34.0;
    draw_text("Credits", x + 18.0, yy, 34.0, Color::from_rgba(210, 220, 255, 255));
    let to_right = x + 120.0;
    draw_text(
        "Controls: F2 closes this page.",
        to_right + 18.0,
        yy,
        20.0,
        Color::from_rgba(180, 200, 230, 255),
    );
    yy += 34.0;
    draw_text(
        "Gameplay and code: Maze Game team",
        x + 18.0,
        yy,
        24.0,
        Color::from_rgba(200, 220, 200, 255),
    );
    yy += 34.0;
    draw_text(
        "Image assets used in this stage:",
        x + 18.0,
        yy,
        22.0,
        Color::from_rgba(230, 230, 230, 255),
    );
    yy += 28.0;
    draw_text(
        "- mazeexitstar.png",
        x + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );
    yy += 22.0;
    draw_text(
        "  source: soulofkiran.itch.io (author: Narik)",
        x + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );
    yy += 28.0;
    draw_text(
        "- hintitem.png",
        x + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );
    yy += 22.0;
    draw_text(
        "  source: twiceuponatime.itch.io (author: TwiceUponATime)",
        x + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );
    yy += 28.0;
    draw_text(
        "- mapimage.png",
        x + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );
    yy += 22.0;
    draw_text(
        "  source: opengameart.org (author: cron)",
        x + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );
    yy += 34.0;
    draw_text(
        "Audio assets used:",
        x + 18.0,
        yy,
        22.0,
        Color::from_rgba(230, 230, 230, 255),
    );
    yy += 28.0;
    let level = yy;
    draw_text(
        "- click_sound.wav",
        x + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );
    yy += 22.0;
    draw_text(
        "   source: pixabay.com (author: CreatorsHome)",
        x + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );
    yy += 28.0;
    draw_text(
        "- exit_found_sound.wav",
        x + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );
    yy += 22.0;
    draw_text(
        "   source: pixabay.com (author: BenKirb)",
        x + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );
    yy += 28.0;
    draw_text(
        "- footstep_1.wav, footstep_2.wav, footstep_3.wav",
        x + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );
    yy += 22.0;
    draw_text(
        "   source: pixabay.com (author: Data_pion)",
        x + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );
    yy += 28.0;
    draw_text(
        "- hint_sound.wav",
        x + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );
    yy += 22.0;
    draw_text(
        "   source: pixabay.com (author: freesound_community)",
        x + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );

    
    let xx = x + 450.0;
    yy = level;
    draw_text(
        "- rain_sound.wav",
        xx + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );
    yy += 22.0;
    draw_text(
        "   source: pixabay.com (author: DRAGON-STUDIO)",
        xx + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );
    yy += 28.0;
    draw_text(
        "- maze_music.wav, menu_music.wav, paper_sound.wav",
        xx + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );
    yy += 22.0;
    draw_text(
        "   source: Maze Game team",
        xx + 24.0,
        yy,
        20.0,
        Color::from_rgba(185, 205, 255, 255),
    );
}
