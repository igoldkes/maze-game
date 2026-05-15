//! Main game coordinator: Macroquad state, update/draw orchestration, and submodules.
use std::sync::atomic::{AtomicBool, Ordering};

static NORMAL_F1_INPUT_QUEUE_CLEARED: AtomicBool = AtomicBool::new(false);
static STARTUP_DEV_PASSWORD_INPUT_QUEUE_CLEARED: AtomicBool = AtomicBool::new(false);
static STARTUP_NAME_INPUT_QUEUE_CLEARED: AtomicBool = AtomicBool::new(false);

mod account_management;
mod assets;
mod mode;
mod player;
mod progress;
mod render;
mod screens;
mod story;
mod ui;
mod world_render;

use crate::maze::{
    center_cell, exit_farthest_on_perimeter, shortest_path_len, Maze,
};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use macroquad::audio::{load_sound, play_sound, play_sound_once, stop_sound, PlaySoundParams, Sound};
use account_management::next_stage_to_play_after_clears;
use progress::{ProgressRecord, ProgressService};
use std::collections::HashSet;
use std::time::Instant;

use player::Player;
use render::{draw_maze_floors, draw_maze_walls, FloorDraw};
use screens::credits_ui::draw_credits_overlay;
use screens::overlays_ui::{
    draw_end_menu_overlay, draw_normal_f1_password_overlay, draw_quit_confirm_overlay,
    draw_unsaved_quit_confirm_overlay,
};
use screens::startup_ui::draw_startup_overlay;
use story::{
    draw_name_prompt, draw_phase_banner, draw_story_prompt, name_on_map_input_ready, opening_story_lines,
    StoryPhase,
};
use ui::{draw_fullscreen_opaque_black, draw_panel, PanelStyle};

const DEV_PASSWORD: &str = "A1B2C3D4";

/// Where startup Esc-confirm should return to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StartupPendingBack {
    ToSplash,
    /// Return to role menu (e.g. from New/Continue or dev password).
    ToAskPlayerRole,
    /// Return to role menu from the records list.
    ToAskPlayerRoleFromRecords,
    ToAskNewOrContinue,
    /// Re-open nickname entry (e.g. back from continue prompt).
    ToAskPlayerName,
    /// Return to continue Y/N after nickname-must-change notice.
    ToAskContinueFromLog,
}

#[derive(Clone, Debug)]
struct HintItem {
    cell: (usize, usize),
    collected: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EndMenuState {
    Hidden,
    /// Exit reached: congratulations + replay vs main menu (no nested save prompts).
    WinCelebration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StartupState {
    /// Initial title splash screen.
    Splash,
    AskPlayerRole,
    /// Choose whether to start a fresh run or continue from local save logs.
    AskNewOrContinue,
    /// Alphanumeric nickname lookup used by the continue flow.
    AskPlayerName,
    /// Prior clears exist for this nickname — offer to continue.
    AskContinueFromLog,
    /// Continue selected, but this nickname has no prior saved clear records.
    ContinueNoRecordNotice,
    /// Player declined continue; must pick another nickname.
    NicknameMustChangeNotice,
    AskDevPassword,
    /// Saved-run list (only opened from main menu after at least one save exists).
    ViewRecords,
    Done,
}

#[derive(Clone, Debug)]
enum RunContext {
    Normal,
    Replay {
        source_record_id: String,
        baseline_secs: f32,
    },
}

enum MusicTrack {
    None,
    Menu,
    Maze,
}

pub struct GameState {
    maze: Maze,
    cols: usize,
    rows: usize,
    stage: usize,
    cell_size: f32,
    origin: Vec2,
    player: Player,
    start_cell: (usize, usize),
    exit_cell: (usize, usize),
    hints: Vec<HintItem>,
    hint_map_timer: f32,
    story: StoryPhase,
    end_menu: EndMenuState,
    startup: StartupState,
    password_buffer: String,
    show_credits: bool,
    easy_test_map: bool,
    test_mask_enabled: bool,
    player_name: String,
    pub debug_overlay: bool,
    path_steps: Option<usize>,
    exit_star: Texture2D,
    hint_icon: Texture2D,
    map_paper: Texture2D,
    hint_sound: Sound,
    exit_found_sound: Sound,
    paper_sound: Sound,
    rain_sound: Sound,
    click_sound: Sound,
    intro_clicked: bool,
    explain_map_clicked: bool,
    map_reveal_clicked: bool,
    controls_clicked: bool,
    playing_clicked: bool,
    maze_music: Sound,
    menu_music: Sound,
    music_track: MusicTrack,
    is_map_open: bool,
    map_sound_played: bool,
    old_buffer_len: usize,
    progress: ProgressService,
    /// True: show “return to main menu?” (Y/N) after Esc.
    quit_confirm: bool,
    /// Second confirmation for unfinished normal-mode stages.
    quit_unsaved_confirm: bool,
    /// When the player is first in `Playing` for this maze, we start this clock for `elapsed_secs` on save.
    play_timer_start: Option<Instant>,
    /// `"prim"` or `"dfs"` for the maze currently loaded (written on progress save for replay tooling).
    maze_generator: String,
    /// Scroll offset in [`StartupState::ViewRecords`] (lines from top of the visible window).
    startup_records_scroll: usize,
    /// Selected row in the startup records list (absolute index in newest-first list).
    startup_records_selected: usize,
    startup_player_name_buffer: String,
    /// Next stage offered when continuing (e.g. cleared 5 → 6).
    startup_continue_next_stage: Option<usize>,
    /// Highest stage previously cleared for the current nickname (for UI text).
    startup_continue_max_cleared: Option<usize>,
    /// Arrow-menu highlight: `Are you a player?` (0 = player, 1 = dev, 2 = records if present).
    startup_menu_role: usize,
    /// Arrow-menu highlight: new vs continue.
    startup_menu_run_type: usize,
    /// Arrow-menu highlight on continue-from-log Y/N.
    startup_menu_continue: usize,
    startup_back_confirm: bool,
    startup_pending_back: Option<StartupPendingBack>,
    /// Elapsed play time **frozen at the moment the exit was reached** (so win-menu idle time is not counted).
    cached_run_elapsed_secs: Option<f32>,
    /// After a win, we append to `player_progress.jsonl` at most once when the player picks an option.
    win_run_saved_to_log: bool,
    /// Win overlay: 0 = play next stage, 1 = restart maze, 2 = main menu.
    win_menu_selection: usize,
    /// Normal-mode developer debug access (unlocked by F1 password prompt).
    normal_dev_authenticated: bool,
    /// Normal-mode debug F3 option: force full-map view while playing.
    normal_dev_f3_full_map: bool,
    /// Modal password prompt for normal-mode F1 debug access.
    normal_f1_password_prompt: bool,
    normal_f1_password_buffer: String,
    normal_f1_password_error: bool,
    run_context: RunContext,
}

impl GameState {
    pub async fn new() -> Self {
        let cols = 31usize;
        let rows = 21usize;
        let hat = assets::build_player_hat_texture();
        let exit_star = assets::load_exit_star_texture("assets/graphics_assets/mazeexitstar.png");
        let hint_icon = assets::load_hint_item_texture("assets/graphics_assets/hintitem.png");
        let map_paper = assets::load_map_paper_texture("assets/graphics_assets/mapimage.png");

        let hint_sound = load_sound("assets/audio_assets/hint_sound.wav").await.unwrap();
        let exit_found_sound = load_sound("assets/audio_assets/exit_found_sound.wav").await.unwrap();
        let paper_sound = load_sound("assets/audio_assets/paper_sound.wav").await.unwrap();
        let rain_sound = load_sound("assets/audio_assets/rain_sound.wav").await.unwrap();
        let click_sound = load_sound("assets/audio_assets/click_sound.wav").await.unwrap();
        let maze_music = load_sound("assets/audio_assets/maze_music.wav").await.unwrap();
        let menu_music = load_sound("assets/audio_assets/menu_music.wav").await.unwrap();
        let music_track = MusicTrack::None;

        let player = Player::new_at_cell_center(vec2(0.0, 0.0), 1.0, 0, 0, hat, 1).await;
        let mut s = Self {
            maze: Maze::generate(1, 1),
            cols,
            rows,
            stage: 1,
            cell_size: 1.0,
            origin: vec2(0.0, 0.0),
            player,
            start_cell: (0, 0),
            exit_cell: (0, 0),
            hints: vec![],
            hint_map_timer: 0.0,
            story: StoryPhase::new_run(),
            end_menu: EndMenuState::Hidden,
            startup: StartupState::Splash,
            password_buffer: String::new(),
            show_credits: false,
            easy_test_map: false,
            test_mask_enabled: true,
            player_name: String::new(),
            debug_overlay: false,
            path_steps: None,
            exit_star,
            hint_icon,
            map_paper,
            hint_sound,
            exit_found_sound,
            paper_sound,
            rain_sound,
            click_sound,
            intro_clicked: false,
            explain_map_clicked: false,
            map_reveal_clicked: false,
            controls_clicked: false,
            playing_clicked: false,
            maze_music,
            menu_music,
            music_track,
            is_map_open: false,
            map_sound_played: false,
            old_buffer_len: 0,
            progress: ProgressService::new_default(),
            quit_confirm: false,
            quit_unsaved_confirm: false,
            play_timer_start: None,
            maze_generator: "prim".to_string(),
            startup_records_scroll: 0,
            startup_records_selected: 0,
            startup_player_name_buffer: String::new(),
            startup_continue_next_stage: None,
            startup_continue_max_cleared: None,
            startup_menu_role: 0,
            startup_menu_run_type: 0,
            startup_menu_continue: 0,
            startup_back_confirm: false,
            startup_pending_back: None,
            cached_run_elapsed_secs: None,
            win_run_saved_to_log: false,
            win_menu_selection: 0,
            normal_dev_authenticated: false,
            normal_dev_f3_full_map: false,
            normal_f1_password_prompt: false,
            normal_f1_password_buffer: String::new(),
            normal_f1_password_error: false,
            run_context: RunContext::Normal,
        };
        s.setup_stage(cols, rows);
        s
    }

    pub fn update(&mut self, dt: f32) {
        if matches!(self.story, StoryPhase::IntroThought) && !self.intro_clicked {
            //play_sound_once(&self.click_sound);
            self.intro_clicked = true;
        }

        if matches!(self.story, StoryPhase::IntroExplainAskMap { .. }) && !self.explain_map_clicked {
            play_sound_once(&self.click_sound);
            self.explain_map_clicked = true;
        }

        if matches!(self.story, StoryPhase::MapReveal { .. }) && !self.map_reveal_clicked {
            play_sound_once(&self.click_sound);
            self.map_reveal_clicked = true;
        }

        if matches!(self.story, StoryPhase::ControlsPrompt) && !self.controls_clicked {
            play_sound_once(&self.click_sound);
            self.controls_clicked = true;
        }

        if matches!(self.story, StoryPhase::Playing) && !self.playing_clicked {
            play_sound_once(&self.click_sound);
            self.playing_clicked = true;
        }

        if matches!(self.story, StoryPhase::IntroThought) && !matches!(self.music_track, MusicTrack::Menu){
            stop_sound(&self.maze_music);
            play_sound(
                &self.menu_music,
                PlaySoundParams {
                    looped: true,
                    volume: 1.,
                },
            );
            self.music_track = MusicTrack::Menu;
        }

        if matches!(self.music_track, MusicTrack::None) {
            play_sound(
                &self.menu_music,
                PlaySoundParams {
                    looped: true,
                    volume: 1.,
                },
            );
            self.music_track = MusicTrack::Menu;
        }

        if matches!(self.music_track, MusicTrack::Menu) && matches!(self.story, StoryPhase::Playing) {
            stop_sound(&self.menu_music);
            play_sound(
                &self.maze_music,
                PlaySoundParams {
                    looped: true,
                    volume: 1.,
                },
            );
            play_sound(
                &self.rain_sound,
                PlaySoundParams {
                    looped: true,
                    volume: 0.4,
                },
            );
            self.music_track = MusicTrack::Maze;
        }

        if !matches!(self.story, StoryPhase::Playing | StoryPhase::Won) {
            stop_sound(&self.rain_sound);
        }
            
        if matches!(self.story, StoryPhase::MapReveal { .. } | StoryPhase::AskNameOnMap { .. }) {
            self.is_map_open = true;
        } else if !self.is_map_open && self.map_sound_played {
            self.map_sound_played = false;
        } else {
            self.is_map_open = false;
        }

        if self.is_map_open && !self.map_sound_played {
            play_sound_once(&self.paper_sound);
            self.map_sound_played = true;
        }

        if self.startup != StartupState::Done {
            self.handle_startup_input();
            return;
        }

        if self.quit_unsaved_confirm {
            if is_key_pressed(KeyCode::Y) || is_key_pressed(KeyCode::Enter) {
                play_sound_once(&self.click_sound);
                self.return_to_main_lobby();
            } else if is_key_pressed(KeyCode::N) || is_key_pressed(KeyCode::Escape) {
                play_sound_once(&self.click_sound);
                self.quit_unsaved_confirm = false;
                self.quit_confirm = false;
            } else if is_key_pressed(KeyCode::R) {
                play_sound_once(&self.click_sound);
                self.restart_level();
            }
            return;
        }

        if self.quit_confirm {
            if is_key_pressed(KeyCode::Y) || is_key_pressed(KeyCode::Enter) {
                play_sound_once(&self.click_sound);
                if self.should_warn_unsaved_quit() {
                    self.quit_unsaved_confirm = true;
                } else {
                    self.return_to_main_lobby();
                }
            } else if is_key_pressed(KeyCode::N) || is_key_pressed(KeyCode::Escape) {
                play_sound_once(&self.click_sound);
                self.quit_confirm = false;
            } else if is_key_pressed(KeyCode::R) {
                play_sound_once(&self.click_sound);
                self.restart_level();
            }
            return;
        }

        // Keep win overlay/input state tied to the Won phase only (avoids stray menu keys mid-intro).
        if !matches!(self.story, StoryPhase::Won) {
            self.end_menu = EndMenuState::Hidden;
        }

        if is_key_pressed(KeyCode::F1) {
            play_sound_once(&self.click_sound);
            if self.easy_test_map || self.normal_dev_authenticated {
                self.debug_overlay = !self.debug_overlay;
            } else {
                self.open_normal_f1_password_prompt();
            }
        }
        if !self.easy_test_map
            && self.normal_dev_authenticated
            && self.debug_overlay
            && is_key_pressed(KeyCode::F3)
        {
            self.normal_dev_f3_full_map = !self.normal_dev_f3_full_map;
        }
        let credits_key_pressed = credits_toggle_pressed();
        if credits_key_pressed {
            self.show_credits = !self.show_credits;
        }
        if self.easy_test_map && is_key_pressed(KeyCode::F4) {
            self.test_mask_enabled = !self.test_mask_enabled;
        }
        if self.show_credits {
            // Prevent "open + close in same frame" when F2 toggles credits on.
            if is_key_pressed(KeyCode::Escape) || (!credits_key_pressed && credits_toggle_pressed()) {
                play_sound_once(&self.click_sound);
                self.show_credits = false;
            }
            return;
        }
        if self.normal_f1_password_prompt {
            self.handle_normal_f1_password_input();
            return;
        }
        if is_key_pressed(KeyCode::Escape) {
            play_sound_once(&self.click_sound);
            match self.end_menu {
                EndMenuState::Hidden => {
                    self.quit_confirm = true;
                    return;
                }
                EndMenuState::WinCelebration => {
                    self.persist_win_run_once();
                    self.return_to_main_lobby();
                    return;
                }
            }
        }
        if is_key_pressed(KeyCode::R) && self.end_menu != EndMenuState::WinCelebration {
            play_sound_once(&self.click_sound);
            self.restart_level();
        }

        if self.hint_map_timer > 0.0 {
            self.hint_map_timer = (self.hint_map_timer - dt).max(0.0);
        }

        // Nickname is now collected on startup; skip the old in-story name prompt when present.
        if !self.player_name.trim().is_empty() && matches!(self.story, StoryPhase::AskNameOnMap { .. }) {
            self.story = StoryPhase::ControlsPrompt;
        }

        if let StoryPhase::AskNameOnMap { buffer, .. } = &mut self.story {
            let new_buffer_len = buffer.len();
            if self.old_buffer_len != new_buffer_len {
                play_sound_once(&self.click_sound);
                self.old_buffer_len = new_buffer_len;
            }
        }

        if let Some(name) = self.story.update(dt) {
            self.player_name = name;
        }
        if self.easy_test_map && !matches!(self.story, StoryPhase::Playing | StoryPhase::Won) {
            self.story = StoryPhase::Playing;
        }

        if matches!(self.story, StoryPhase::Playing) && self.play_timer_start.is_none() {
            self.play_timer_start = Some(Instant::now());
        }

        if self.can_move_now() {
            self.player
                .update(&self.maze, self.origin, self.cell_size, dt);
        }

        if matches!(self.story, StoryPhase::Playing) {
            let (px, py) = self.player.current_cell(self.origin, self.cell_size, self.cols, self.rows);
            for hint in &mut self.hints {
                if !hint.collected && hint.cell == (px, py) {
                    play_sound_once(&self.hint_sound);
                    hint.collected = true;
                    self.hint_map_timer = 5.0;
                }
            }
            if (px, py) == self.exit_cell {
                play_sound_once(&self.exit_found_sound);
                self.cached_run_elapsed_secs = self
                    .play_timer_start
                    .map(|t| t.elapsed().as_secs_f32());
                self.story.set_won();
                self.intro_clicked = false;
                self.explain_map_clicked = false;
                self.map_reveal_clicked = false;
                self.controls_clicked = false;
                self.playing_clicked = false;
                self.win_run_saved_to_log = false;
                self.win_menu_selection = 0;
                self.end_menu = EndMenuState::WinCelebration;
            }
        }
        self.handle_end_menu_input();
    }

    pub fn restart_level(&mut self) {
        Player::respawn_at_cell(&mut self.player, self.origin, self.cell_size, self.start_cell.0, self.start_cell.1);
        self.play_timer_start = Some(Instant::now());
        self.end_menu = EndMenuState::Hidden;
        let full_map_visible = self.full_map_visible_now();
        let vis = world_render::visibility_grid(self.cols, self.rows, self.start_cell, self.cell_size);
        for hint in &mut self.hints {
            if hint.collected {
                hint.collected = false;
            }
        }
        self.draw_hints(self.origin, self.cell_size, full_map_visible == false, &vis);
        self.story = StoryPhase::Restart;
    }

    pub fn draw(&self) {
        if self.startup != StartupState::Done {
            draw_startup_overlay(
                &self.startup,
                &self.progress,
                &self.password_buffer,
                self.startup_records_scroll,
                self.startup_records_selected,
                &self.player_name,
                &self.startup_player_name_buffer,
                STARTUP_NAME_INPUT_QUEUE_CLEARED.load(Ordering::Relaxed),
                self.startup_continue_next_stage,
                self.startup_continue_max_cleared,
                self.startup_menu_role,
                self.startup_menu_run_type,
                self.startup_menu_continue,
                self.startup_back_confirm,
            );
            return;
        }
        let floor = Color::from_rgba(35, 40, 55, 255);

        let cols = self.cols;
        let rows = self.rows;
        let paper_intro = matches!(
            self.story,
            StoryPhase::MapReveal { .. } | StoryPhase::AskNameOnMap { .. }
        );
        let full_map_visible = self.full_map_visible_now();
        let player_cell = self.player.current_cell(self.origin, self.cell_size, self.cols, self.rows);
        let use_camera = self.use_camera_mask() && !paper_intro;
        if !paper_intro {
            if use_camera {
                let cam = self.build_player_camera(5.0);
                set_camera(&cam);
            }
            // True circular mask: draw only cells in radius unless temporarily revealing full map.
            let visibility_full = if self.easy_test_map && !self.test_mask_enabled {
                true
            } else {
                full_map_visible
            };
            self.draw_world_with_visibility(floor, player_cell, 5.0, visibility_full);
            self.player.draw();
            // Mask must be drawn in **screen space** (default camera). While the player camera is
            // active, draw_rectangle uses world coordinates and would mis-place the vignette.
            set_default_camera();
            if use_camera {
                self.draw_screen_circle_mask(5.0);
            }
        } else {
            set_default_camera();
            draw_fullscreen_opaque_black();
            self.draw_map_paper_overlay();
        }

        if self.story.show_map_item() {
            self.draw_folded_map_item();
        }

        if let StoryPhase::IntroThought = &self.story {
            let line = opening_story_lines()[0];
            draw_story_prompt(line, "", "Press Space or Enter");
        } else if let StoryPhase::IntroExplainAskMap { is_first_stage } = &self.story {
            if *is_first_stage {
                draw_story_prompt(
                    "You are inside a maze.",
                    "There is something in your pocket: a map.",
                    "Press Space to view the map",
                );
            } else {
                draw_story_prompt(
                    "You find yourself inside another maze.",
                    "There is another map in your pocket.",
                    &format!("Press Space to view the map for Stage {0}", self.stage),
                );
            }
        } else if let StoryPhase::AskNameOnMap { buffer, .. } = &self.story {
            draw_name_prompt(buffer, name_on_map_input_ready());
        } else if let StoryPhase::ControlsPrompt = &self.story {
            draw_story_prompt(
                "Use WASD or Arrow keys to move through the maze.\tPress R to replay.",
                "Hint: collect hint maps to reveal the full maze briefly.",
                "Press Enter to start",
            );
        }

        draw_phase_banner(&self.story, &self.player_name);

        let mut hud = format!(
            "Stage {} · Maze {}×{} · FPS {} · Player {}",
            self.stage,
            cols,
            rows,
            get_fps(),
            if self.player_name.trim().is_empty() {
                "Traveler"
            } else {
                self.player_name.trim()
            }
        );
        if self.hint_map_timer > 0.0 {
            hud.push_str(&format!(" · full map {:.0}s", self.hint_map_timer.ceil()));
        }
        if self.debug_overlay {
            if let Some(steps) = self.path_steps {
                hud.push_str(&format!(
                    " · steps {}→exit: {}",
                    format_cell(self.start_cell),
                    steps
                ));
            } else {
                hud.push_str(" · unreachable (bug)");
            }
        }
        let hud_w = (hud.len() as f32 * 10.3).min(screen_width() - 20.0);
        draw_panel(
            Rect::new(8.0, 6.0, hud_w, 26.0),
            PanelStyle {
                bg: Color::from_rgba(8, 10, 20, 190),
                border: None,
            },
        );
        draw_text(&hud, 10.0, 24.0, 22.0, Color::from_rgba(160, 220, 160, 255));

        if self.debug_overlay {
            draw_panel(
                Rect::new(8.0, 34.0, 730.0, 22.0),
                PanelStyle {
                    bg: Color::from_rgba(8, 10, 20, 170),
                    border: None,
                },
            );
            draw_text(
                "WASD / arrows move grid-by-grid · collect cyan hints for 5s full map",
                10.0,
                50.0,
                18.0,
                Color::from_rgba(140, 160, 200, 255),
            );
        }
        draw_panel(
            Rect::new(8.0, screen_height() - 28.0, 560.0, 22.0),
            PanelStyle {
                bg: Color::from_rgba(8, 10, 20, 170),
                border: None,
            },
        );
        let footer = if self.easy_test_map {
            "Esc: main menu (confirm) · [F4] visibility mask · [F1] debug"
        } else if self.normal_dev_authenticated && self.debug_overlay {
            if self.normal_dev_f3_full_map {
                "Esc: main menu (confirm) · [F1] debug · [F3] full-map ON · [F2] credits"
            } else {
                "Esc: main menu (confirm) · [F1] debug · [F3] full-map OFF · [F2] credits"
            }
        } else {
            "Esc: main menu (confirm) · [F1] dev debug (password) · [F2] credits"
        };
        draw_text(
            footer,
            12.0,
            screen_height() - 11.0,
            21.0,
            Color::from_rgba(210, 225, 255, 255),
        );

        if self.end_menu == EndMenuState::WinCelebration {
            let replay_cmp = self.replay_result_line();
            draw_end_menu_overlay(
                &self.player_name,
                replay_cmp.as_ref().map(|(msg, color)| (msg.as_str(), *color)),
                self.cached_run_elapsed_secs,
                self.win_menu_selection,
            );
        }
        draw_credits_overlay(self.show_credits);
        if self.easy_test_map {
            self.draw_test_mode_shortcuts();
        }
        if self.quit_confirm {
            draw_quit_confirm_overlay();
        }
        if self.quit_unsaved_confirm {
            draw_unsaved_quit_confirm_overlay();
        }
        if self.normal_f1_password_prompt {
            draw_normal_f1_password_overlay(
                &self.normal_f1_password_buffer,
                self.normal_f1_password_error,
            );
        }
        // if self.replay {

        // }
    }

    fn draw_folded_map_item(&self) {
        let cols = self.cols;
        let rows = self.rows;
        // 150% larger than the current 480x400 target.
        let max_w = 720.0;
        let max_h = 600.0;
        let aspect = cols as f32 / rows as f32;
        let (box_w, box_h) = if aspect >= max_w / max_h {
            (max_w, max_w / aspect)
        } else {
            (max_h * aspect, max_h)
        };

        let ox = (screen_width() - box_w) * 0.5;
        let oy = (screen_height() - box_h) * 0.5;

        draw_panel(
            Rect::new(ox - 6.0, oy - 28.0, box_w + 12.0, box_h + 40.0),
            PanelStyle {
                bg: Color::from_rgba(8, 10, 18, 255),
                border: None,
            },
        );
        draw_text(
            "Map of the maze",
            ox,
            oy - 12.0,
            18.0,
            Color::from_rgba(200, 210, 240, 255),
        );

        let cw = box_w / cols as f32;
        let ch = box_h / rows as f32;
        let cc = cw.min(ch);
        let map_w = cc * cols as f32;
        let map_h = cc * rows as f32;
        let mx = ox + (box_w - map_w) / 2.0;
        let my = oy + (box_h - map_h) / 2.0;

        let floor = Color::from_rgba(40, 45, 60, 255);
        let m = &self.maze;
        draw_maze_floors(&FloorDraw {
            origin: vec2(mx, my),
            cell_size: cc,
            cols,
            rows,
            floor_color: floor,
            exit_cell: self.exit_cell,
            show_exit_tint: true,
        });
        draw_maze_walls(vec2(mx, my), cc, cols, rows, |x, y| m.walls(x, y).0);
        //self.draw_exit_star(vec2(mx, my), cc);
        let vis_all = vec![true; cols * rows];
        self.draw_hints(vec2(mx, my), cc, true, &vis_all);

        let (px, py) = self.player.current_cell(self.origin, self.cell_size, cols, rows);
        let pcx = mx + px as f32 * cc + cc * 0.5;
        let pcy = my + py as f32 * cc + cc * 0.5;
        // Keep player marker consistent with the in-world yellow marker.
        draw_circle(pcx, pcy, cc * 0.18, Color::from_rgba(255, 225, 75, 255));

        let (ex, ey) = self.exit_cell;
        let ecx = mx + ex as f32 * cc + cc * 0.5;
        let ecy = my + ey as f32 * cc + cc * 0.5;
        draw_circle(ecx, ecy, cc * 0.18, Color::from_rgba(255, 100, 100, 255));
        self.draw_exit_star(vec2(mx, my), cc);
    }

    fn draw_world_with_visibility(
        &self,
        floor: Color,
        center: (usize, usize),
        radius_cells: f32,
        full_map_visible: bool,
    ) {
        let cols = self.cols;
        let rows = self.rows;
        let cs = self.cell_size;
        let vis = world_render::visibility_grid(cols, rows, center, radius_cells);
        // Black out entire world view; visible cells draw over it.
        draw_rectangle(
            self.origin.x - cs * 2.0,
            self.origin.y - cs * 2.0,
            cols as f32 * cs + cs * 4.0,
            rows as f32 * cs + cs * 4.0,
            BLACK,
        );
        for y in 0..rows {
            for x in 0..cols {
                if !full_map_visible && !world_render::is_visible(&vis, cols, x, y) {
                    continue;
                }
                let px = self.origin.x + x as f32 * cs;
                let py = self.origin.y + y as f32 * cs;
                let mut c = floor;
                if full_map_visible && (x, y) == self.exit_cell {
                    c = Color::from_rgba(90, 50, 55, 255);
                }
                draw_rectangle(px, py, cs, cs, c);
            }
        }
        let m = &self.maze;
        draw_maze_walls(self.origin, cs, cols, rows, |x, y| {
            if full_map_visible || world_render::is_visible(&vis, cols, x, y) {
                m.walls(x, y).0
            } else {
                0
            }
        });
        if full_map_visible || world_render::is_visible(&vis, cols, self.exit_cell.0, self.exit_cell.1) {
            self.draw_exit_star(self.origin, cs);
        }
        self.draw_hints(self.origin, cs, full_map_visible, &vis);
    }

    fn draw_hints(&self, draw_origin: Vec2, cell_size: f32, full_map_visible: bool, vis: &[bool]) {
        for hint in &self.hints {
            if hint.collected {
                continue;
            }
            if !full_map_visible
                && !world_render::is_visible(vis, self.cols, hint.cell.0, hint.cell.1)
            {
                continue;
            }
            let size = cell_size * 0.8;
            let x = draw_origin.x + hint.cell.0 as f32 * cell_size + (cell_size - size) * 0.8;
            let y = draw_origin.y + hint.cell.1 as f32 * cell_size + (cell_size - size) * 0.8;
            draw_texture_ex(
                &self.hint_icon,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(size, size)),
                    ..Default::default()
                },
            );
        }
    }

    fn draw_exit_star(&self, draw_origin: Vec2, cell_size: f32) {
        let (ex, ey) = self.exit_cell;
        let size = cell_size * 0.45;
        let x = draw_origin.x + ex as f32 * cell_size + (cell_size - size) * 0.5;
        let y = draw_origin.y + ey as f32 * cell_size + (cell_size - size) * 0.5;
        draw_texture_ex(
            &self.exit_star,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                ..Default::default()
            },
        );
    }

    fn build_player_camera(&self, view_radius_cells: f32) -> Camera2D {
        let cells_diameter = view_radius_cells * 2.0 + 1.0;
        let view_w = cells_diameter * self.cell_size;
        let aspect = screen_height() / screen_width();
        let view_h = view_w * aspect;
        // Negative height keeps world Y increasing downward so up/down controls align visually.
        Camera2D::from_display_rect(Rect::new(
            self.player.pos.x - view_w * 0.5,
            self.player.pos.y + view_h * 0.5,
            view_w,
            -view_h,
        ))
    }

    /// Screen-space rectangle where the parchment texture is drawn (map reveal intro).
    fn map_paper_dest_rect(&self) -> Rect {
        let w = screen_width();
        let h = screen_height();
        let tex_sz = self.map_paper.size();
        // 30% larger than previous 70% target => 91% of screen bounds.
        let target_w = w * 0.91;
        let target_h = h * 0.91;
        let scale = (target_w / tex_sz.x).min(target_h / tex_sz.y);
        let draw_w = tex_sz.x * scale;
        let draw_h = tex_sz.y * scale;
        let x = (w - draw_w) * 0.5;
        let y = (h - draw_h) * 0.5;
        Rect::new(x, y, draw_w, draw_h)
    }

    fn draw_map_paper_overlay(&self) {
        let r = self.map_paper_dest_rect();
        draw_texture_ex(
            &self.map_paper,
            r.x,
            r.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(r.w, r.h)),
                ..Default::default()
            },
        );
    }

    /// Circular blackout vignette in **screen pixels**. Caller must use the default (screen) camera.
    /// Hole stays at the viewport center; the player camera keeps the avatar centered so this matches
    /// a fixed “porthole” over a scrolling world.
    fn draw_screen_circle_mask(&self, view_radius_cells: f32) {
        let w = screen_width();
        let h = screen_height();
        let cx = w * 0.5;
        let cy = h * 0.5;
        // Match world visibility radius to screen: camera maps `cells_diameter * cell_size` wide to `w`.
        let cells_diameter = view_radius_cells * 2.0 + 1.0;
        let view_w_world = cells_diameter * self.cell_size;
        let mut r = if view_w_world > f32::EPSILON {
            view_radius_cells * self.cell_size * (w / view_w_world)
        } else {
            w * 0.25
        };
        r = (r * 1.02).clamp(48.0, w.min(h) * 0.49);
        let step = 2.0_f32;
        let mut y = 0.0_f32;
        while y < h {
            let dy = y + step * 0.5 - cy;
            if dy.abs() > r {
                draw_rectangle(0.0, y, w, step, BLACK);
            } else {
                let inside = (r * r - dy * dy).sqrt();
                let lx = (cx - inside).max(0.0);
                let rx = (cx + inside).min(w);
                if lx > 0.0 {
                    draw_rectangle(0.0, y, lx, step, BLACK);
                }
                if rx < w {
                    draw_rectangle(rx, y, w - rx, step, BLACK);
                }
            }
            y += step;
        }
    }

    fn persist_win_run_once(&mut self) {
        if self.win_run_saved_to_log {
            return;
        }
        if self.easy_test_map {
            // Test/dev map completions are intentionally excluded from player progress logs.
            self.win_run_saved_to_log = true;
            return;
        }
        self.save_current_success();
        self.win_run_saved_to_log = true;
    }

    fn handle_end_menu_input(&mut self) {
        if !matches!(self.story, StoryPhase::Won) {
            return;
        }
        match self.end_menu {
            EndMenuState::Hidden => {}
            EndMenuState::WinCelebration => {
                if is_key_pressed(KeyCode::Up) {
                    if self.win_menu_selection == 0 {
                        play_sound_once(&self.click_sound);
                        self.win_menu_selection = 2;
                    } else {
                        play_sound_once(&self.click_sound);
                        self.win_menu_selection = self.win_menu_selection.saturating_sub(1);
                    }
                }
                if is_key_pressed(KeyCode::Down) {
                    if self.win_menu_selection == 2 {
                        play_sound_once(&self.click_sound);
                        self.win_menu_selection = 0;
                    } else {
                        play_sound_once(&self.click_sound);
                        self.win_menu_selection = (self.win_menu_selection + 1).min(2);
                    }
                }
                if is_key_pressed(KeyCode::Enter) {
                    play_sound_once(&self.click_sound);
                    match self.win_menu_selection {
                        0 => {
                            self.persist_win_run_once();
                            self.advance_to_next_stage();
                        }
                        1 => {
                            self.restart_level();
                        }
                        2 => {
                            self.persist_win_run_once();
                            self.return_to_main_lobby();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn replay_result_line(&self) -> Option<(String, Color)> {
        let RunContext::Replay { baseline_secs, .. } = &self.run_context else {
            return None;
        };
        let current = self.cached_run_elapsed_secs.unwrap_or_else(|| {
            self.play_timer_start
                .map(|t| t.elapsed().as_secs_f32())
                .unwrap_or(0.0)
        });
        let delta = current - *baseline_secs;
        if delta < -0.05 {
            Some((
                format!("Replay result: {:.1}s faster than your saved run.", -delta),
                Color::from_rgba(150, 235, 180, 255),
            ))
        } else if delta > 0.05 {
            Some((
                format!("Replay result: {:.1}s slower than your saved run.", delta),
                Color::from_rgba(255, 195, 150, 255),
            ))
        } else {
            Some((
                "Replay result: matched your saved run time.".to_string(),
                Color::from_rgba(225, 225, 190, 255),
            ))
        }
    }

    fn save_current_success(&mut self) {
        let name = if self.player_name.trim().is_empty() {
            "player".to_string()
        } else {
            self.player_name.trim().to_string()
        };
        let elapsed_secs = self.cached_run_elapsed_secs.unwrap_or_else(|| {
            self.play_timer_start
                .map(|t| t.elapsed().as_secs_f32())
                .unwrap_or(0.0)
        });
        let mut record = ProgressRecord::new_run_snapshot(
            name,
            self.stage as u32,
            self.cols,
            self.rows,
            self.start_cell,
            self.exit_cell,
            self.maze_generator.clone(),
            elapsed_secs,
            self.maze.cells().to_vec(),
        );
        if let RunContext::Replay {
            source_record_id,
            baseline_secs,
        } = &self.run_context
        {
            record.is_replay = true;
            record.replay_of_record_id = Some(source_record_id.clone());
            record.baseline_secs = Some(*baseline_secs);
        }
        let _ = self.progress.append_success(&record);
    }

    fn advance_to_next_stage(&mut self) {
        self.run_context = RunContext::Normal;
        self.stage += 1;
        self.player.stage = self.stage;
        // Always derive size from `stage` so the maze cannot drift out of sync with the HUD
        // (e.g. lobby reset to 31×21 while `stage` was still > 1).
        let (cols, rows) = Self::normal_maze_dims_for_stage(self.stage);
        self.setup_stage(cols, rows);
        self.story = StoryPhase::IntroExplainAskMap { is_first_stage: false };
        //self.story = StoryPhase::Playing;
        self.end_menu = EndMenuState::Hidden;
        self.hint_map_timer = 0.0;
    }

    /// Stage-1 normal maze (31×21). Call whenever the player is on the main menu path before a run starts.
    fn reset_normal_lobby_geometry(&mut self) {
        self.stage = 1;
        self.setup_stage(31, 21);
    }

    /// Clear current run state and rebuild defaults when returning to the main lobby.
    fn return_to_main_lobby(&mut self) {
        self.quit_confirm = false;
        self.quit_unsaved_confirm = false;
        self.startup = StartupState::AskPlayerRole;
        self.end_menu = EndMenuState::Hidden;
        self.show_credits = false;
        self.hint_map_timer = 0.0;
        self.player_name.clear();
        self.story = StoryPhase::new_run();
        self.easy_test_map = false;
        self.test_mask_enabled = true;
        self.normal_dev_f3_full_map = false;
        self.normal_f1_password_prompt = false;
        self.normal_f1_password_buffer.clear();
        self.normal_f1_password_error = false;
        self.startup_records_scroll = 0;
        self.startup_records_selected = 0;
        self.startup_player_name_buffer.clear();
        self.startup_continue_next_stage = None;
        self.startup_continue_max_cleared = None;
        self.password_buffer.clear();
        self.run_context = RunContext::Normal;
        self.reset_normal_lobby_geometry();
    }

    /// Normal mode: maze size for a given stage (stage 1 = 31×21; each stage adds 2 to both sides).
    fn normal_maze_dims_for_stage(stage: usize) -> (usize, usize) {
        let s = stage.max(1);
        let add = s.saturating_sub(1) * 2;
        (31 + add, 21 + add)
    }

    /// Leave startup and begin a normal run at `start_stage` (1 = fresh, or continue from a later stage).
    ///
    /// `show_controls_prompt = true` means "fresh game intro flow" (includes the 10s map reveal and
    /// name-on-map dialog via `StoryPhase::new_run()`), not a direct jump into movement.
    fn begin_normal_play_after_startup(&mut self, start_stage: usize, show_controls_prompt: bool) {
        self.easy_test_map = false;
        self.stage = start_stage.max(1);
        self.player.stage = self.stage;
        let (cols, rows) = Self::normal_maze_dims_for_stage(self.stage);
        self.setup_stage(cols, rows);
        self.story = if show_controls_prompt {
            StoryPhase::new_run()
        } else {
            StoryPhase::Playing
        };
        self.startup = StartupState::Done;
        self.startup_player_name_buffer.clear();
        self.startup_continue_next_stage = None;
        self.startup_continue_max_cleared = None;
    }

    fn should_warn_unsaved_quit(&self) -> bool {
        !self.easy_test_map && !matches!(self.story, StoryPhase::Won)
    }

    /// Opens the F1 dev-password panel with a clean buffer and no stale `get_char_pressed` queue.
    fn open_normal_f1_password_prompt(&mut self) {
        self.normal_f1_password_prompt = true;
        self.normal_f1_password_buffer.clear();
        self.normal_f1_password_error = false;
        NORMAL_F1_INPUT_QUEUE_CLEARED.store(false, Ordering::Relaxed);
    }

    fn handle_normal_f1_password_input(&mut self) {
        if !NORMAL_F1_INPUT_QUEUE_CLEARED.load(Ordering::Relaxed) {
            while get_char_pressed().is_some() {}
            NORMAL_F1_INPUT_QUEUE_CLEARED.store(true, Ordering::Relaxed);
            return;
        }
        while let Some(ch) = get_char_pressed() {
            if ch.is_ascii_alphanumeric() && self.normal_f1_password_buffer.len() < 32 {
                play_sound_once(&self.click_sound);
                self.normal_f1_password_buffer.push(ch);
            }
        }
        if is_key_pressed(KeyCode::Backspace) && !self.normal_f1_password_buffer.is_empty() {
            play_sound_once(&self.click_sound);
            self.normal_f1_password_buffer.pop();
        }
        if is_key_pressed(KeyCode::Escape) {
            play_sound_once(&self.click_sound);
            self.normal_f1_password_prompt = false;
            self.normal_f1_password_buffer.clear();
            self.normal_f1_password_error = false;
            return;
        }
        if is_key_pressed(KeyCode::Enter) {
            play_sound_once(&self.click_sound);
            if self.normal_f1_password_buffer == DEV_PASSWORD {
                self.normal_dev_authenticated = true;
                self.debug_overlay = true;
                self.normal_f1_password_error = false;
            } else {
                // Processed wrong submit: close prompt and clear input for safety.
                self.normal_f1_password_error = false;
            }
            self.normal_f1_password_prompt = false;
            self.normal_f1_password_buffer.clear();
        }
    }

    fn setup_stage(&mut self, cols: usize, rows: usize) {
        self.cols = cols;
        self.rows = rows;
        self.play_timer_start = None;
        self.cached_run_elapsed_secs = None;
        self.win_run_saved_to_log = false;
        self.run_context = RunContext::Normal;
        self.maze = if gen_range(0, 2) == 0 {
            self.maze_generator = "prim".to_string();
            Maze::generate(cols, rows)
        } else {
            self.maze_generator = "dfs".to_string();
            Maze::generate_depth_first(cols, rows)
        };
        self.start_cell = center_cell(cols, rows);
        self.exit_cell = exit_farthest_on_perimeter(&self.maze, self.start_cell);
        self.rebuild_world_after_maze_change();
    }

    fn rebuild_world_after_maze_change(&mut self) {
        let cols = self.cols;
        let rows = self.rows;
        self.path_steps = shortest_path_len(&self.maze, self.start_cell, self.exit_cell);

        let margin = 40.0;
        let cw = screen_width() - margin * 2.0;
        let ch = screen_height() - margin * 2.0;
        let cell_w = cw / cols as f32;
        let cell_h = ch / rows as f32;
        self.cell_size = cell_w.min(cell_h);
        let grid_w = self.cell_size * cols as f32;
        let grid_h = self.cell_size * rows as f32;
        self.origin = vec2(
            (screen_width() - grid_w) / 2.0,
            (screen_height() - grid_h) / 2.0,
        );
        self.player
            .respawn_at_cell(self.origin, self.cell_size, self.start_cell.0, self.start_cell.1);
        self.hints = build_hint_cells(cols, rows)
            .into_iter()
            .filter(|&c| c != self.start_cell && c != self.exit_cell)
            .map(|c| HintItem {
                cell: c,
                collected: false,
            })
            .collect();
    }

    fn start_replay_from_record(&mut self, record: ProgressRecord) -> bool {
        let expected_cells = record.maze_w.saturating_mul(record.maze_h);
        if expected_cells == 0 || record.cells.len() != expected_cells {
            return false;
        }
        self.quit_confirm = false;
        self.quit_unsaved_confirm = false;
        self.end_menu = EndMenuState::Hidden;
        self.show_credits = false;
        self.easy_test_map = false;
        self.test_mask_enabled = true;
        self.stage = record.stage as usize;
        self.cols = record.maze_w;
        self.rows = record.maze_h;
        self.start_cell = (record.start_x, record.start_y);
        self.exit_cell = (record.exit_x, record.exit_y);
        self.maze_generator = record.maze_generator.clone();
        self.maze = Maze::new(record.maze_w, record.maze_h, record.cells);
        self.run_context = RunContext::Replay {
            source_record_id: record.record_id,
            baseline_secs: record.elapsed_secs,
        };
        self.play_timer_start = None;
        self.cached_run_elapsed_secs = None;
        self.win_run_saved_to_log = false;
        self.hint_map_timer = 0.0;
        if self.player_name.trim().is_empty() {
            self.player_name = record.player_name;
        }
        self.story = StoryPhase::Playing;
        self.rebuild_world_after_maze_change();
        true
    }

    fn apply_startup_pending_back(&mut self, target: StartupPendingBack) {
        match target {
            StartupPendingBack::ToSplash => {
                self.startup = StartupState::Splash;
            }
            StartupPendingBack::ToAskPlayerRole => {
                self.startup_menu_role = self
                    .startup_menu_role
                    .min(if self.progress.has_saved_records() {
                        2
                    } else {
                        1
                    });
                self.startup = StartupState::AskPlayerRole;
            }
            StartupPendingBack::ToAskPlayerRoleFromRecords => {
                self.reset_normal_lobby_geometry();
                self.startup_records_scroll = 0;
                self.startup_records_selected = 0;
                self.startup_menu_role = self
                    .startup_menu_role
                    .min(if self.progress.has_saved_records() {
                        2
                    } else {
                        1
                    });
                self.startup = StartupState::AskPlayerRole;
            }
            StartupPendingBack::ToAskNewOrContinue => {
                self.startup_player_name_buffer.clear();
                STARTUP_NAME_INPUT_QUEUE_CLEARED.store(false, Ordering::Relaxed);
                self.startup_menu_run_type = 0;
                self.startup = StartupState::AskNewOrContinue;
            }
            StartupPendingBack::ToAskPlayerName => {
                self.startup_player_name_buffer.clear();
                self.startup_player_name_buffer.push_str(&self.player_name);
                self.player_name.clear();
                self.startup_continue_next_stage = None;
                self.startup_continue_max_cleared = None;
                STARTUP_NAME_INPUT_QUEUE_CLEARED.store(false, Ordering::Relaxed);
                self.startup = StartupState::AskPlayerName;
            }
            StartupPendingBack::ToAskContinueFromLog => {
                self.startup_menu_continue = 0;
                self.startup = StartupState::AskContinueFromLog;
            }
        }
    }

    fn request_startup_back(&mut self, target: StartupPendingBack) {
        self.startup_back_confirm = true;
        self.startup_pending_back = Some(target);
    }

    fn handle_startup_input(&mut self) {
        if self.startup_back_confirm {
            if is_key_pressed(KeyCode::Y) || is_key_pressed(KeyCode::Enter) {
                play_sound_once(&self.click_sound);
                self.startup_back_confirm = false;
                if let Some(t) = self.startup_pending_back.take() {
                    self.apply_startup_pending_back(t);
                }
            } else if is_key_pressed(KeyCode::N) || is_key_pressed(KeyCode::Escape) {
                play_sound_once(&self.click_sound);
                self.startup_back_confirm = false;
                self.startup_pending_back = None;
            }
            return;
        }

        match self.startup {
            StartupState::Splash => {
                if get_last_key_pressed().is_some() {
                    play_sound_once(&self.click_sound);
                    self.reset_normal_lobby_geometry();
                    self.startup_menu_role = 0;
                    self.startup = StartupState::AskPlayerRole;
                }
            }
            StartupState::AskPlayerRole => {
                let n_options = if self.progress.has_saved_records() {
                    4
                } else {
                    3
                };
                if is_key_pressed(KeyCode::Up) {
                    if self.startup_menu_role == 0 {
                        play_sound_once(&self.click_sound);
                        self.startup_menu_role = n_options - 1;
                    } else {
                        play_sound_once(&self.click_sound);
                        self.startup_menu_role = self.startup_menu_role.saturating_sub(1);
                    }
                }
                if is_key_pressed(KeyCode::Down) {
                    if self.startup_menu_role == n_options - 1 {
                        play_sound_once(&self.click_sound);
                        self.startup_menu_role = 0;
                    } else {
                        play_sound_once(&self.click_sound);
                        self.startup_menu_role = (self.startup_menu_role + 1).min(n_options - 1);
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    play_sound_once(&self.click_sound);
                    self.request_startup_back(StartupPendingBack::ToSplash);
                    return;
                }
                if is_key_pressed(KeyCode::Enter) {
                    play_sound_once(&self.click_sound);
                    match self.startup_menu_role {
                        0 => {
                            self.easy_test_map = false;
                            self.reset_normal_lobby_geometry();
                            self.player_name.clear();
                            self.startup_player_name_buffer.clear();
                            self.startup_continue_next_stage = None;
                            self.startup_continue_max_cleared = None;
                            self.startup_menu_run_type = 0;
                            self.startup = StartupState::AskNewOrContinue;
                        }
                        1 => {
                            self.password_buffer.clear();
                            STARTUP_DEV_PASSWORD_INPUT_QUEUE_CLEARED.store(false, Ordering::Relaxed);
                            self.startup = StartupState::AskDevPassword;
                        }
                        2 => {
                            self.startup_records_scroll = 0;
                            self.startup_records_selected = 0;
                            self.startup = StartupState::ViewRecords;
                        }
                        3 => {
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
            }
            StartupState::AskNewOrContinue => {
                if is_key_pressed(KeyCode::Up) {
                    if self.startup_menu_run_type == 0 {
                        play_sound_once(&self.click_sound);
                        self.startup_menu_run_type = 1;
                    } else {
                        play_sound_once(&self.click_sound);
                        self.startup_menu_run_type = self.startup_menu_run_type.saturating_sub(1);
                    }
                }
                if is_key_pressed(KeyCode::Down) {
                    if self.startup_menu_run_type == 1 {
                        play_sound_once(&self.click_sound);
                        self.startup_menu_run_type = 0;
                    } else {
                        play_sound_once(&self.click_sound);
                        self.startup_menu_run_type = (self.startup_menu_run_type + 1).min(1);
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    play_sound_once(&self.click_sound);
                    self.request_startup_back(StartupPendingBack::ToAskPlayerRole);
                    return;
                }
                if is_key_pressed(KeyCode::Enter) {
                    play_sound_once(&self.click_sound);
                    if self.startup_menu_run_type == 0 {
                        self.player_name.clear();
                        self.begin_normal_play_after_startup(1, true);
                    } else {
                        self.player_name.clear();
                        self.startup_player_name_buffer.clear();
                        self.startup_continue_next_stage = None;
                        self.startup_continue_max_cleared = None;
                        STARTUP_NAME_INPUT_QUEUE_CLEARED.store(false, Ordering::Relaxed);
                        self.startup = StartupState::AskPlayerName;
                    }
                }
            }
            StartupState::AskPlayerName => {
                if is_key_pressed(KeyCode::Escape) {
                    play_sound_once(&self.click_sound);
                    self.request_startup_back(StartupPendingBack::ToAskNewOrContinue);
                    return;
                }
                if !STARTUP_NAME_INPUT_QUEUE_CLEARED.load(Ordering::Relaxed) {
                    for _ in 0..4096 {
                        if get_char_pressed().is_none() {
                            break;
                        }
                    }
                    STARTUP_NAME_INPUT_QUEUE_CLEARED.store(true, Ordering::Relaxed);
                    return;
                }
                while let Some(ch) = get_char_pressed() {
                    if ch.is_ascii_alphanumeric() && self.startup_player_name_buffer.len() < 32 {
                        play_sound_once(&self.click_sound);
                        self.startup_player_name_buffer.push(ch);
                    }
                }
                if is_key_pressed(KeyCode::Backspace) && !self.startup_player_name_buffer.is_empty() {
                    play_sound_once(&self.click_sound);
                    self.startup_player_name_buffer.pop();
                }
                if is_key_pressed(KeyCode::Enter) {
                    play_sound_once(&self.click_sound);
                    let name = self.startup_player_name_buffer.trim().to_string();
                    if name.is_empty() {
                        return;
                    }
                    self.player_name = name;
                    if let Some(next_stage) =
                        next_stage_to_play_after_clears(&self.progress, &self.player_name)
                    {
                        self.startup_continue_next_stage = Some(next_stage);
                        self.startup_continue_max_cleared = Some(next_stage.saturating_sub(1));
                        self.startup_menu_continue = 0;
                        self.startup = StartupState::AskContinueFromLog;
                    } else {
                        self.startup = StartupState::ContinueNoRecordNotice;
                    }
                }
            }
            StartupState::AskContinueFromLog => {
                if is_key_pressed(KeyCode::Up) {
                    if self.startup_menu_continue == 0 {
                        play_sound_once(&self.click_sound);
                        self.startup_menu_continue = 1;
                    } else {
                        play_sound_once(&self.click_sound);
                        self.startup_menu_continue = self.startup_menu_continue.saturating_sub(1);
                    }
                }
                if is_key_pressed(KeyCode::Down) {
                    if self.startup_menu_continue == 1 {
                        play_sound_once(&self.click_sound);
                        self.startup_menu_continue = 0;
                    } else {
                        play_sound_once(&self.click_sound);
                        self.startup_menu_continue = (self.startup_menu_continue + 1).min(1);
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    play_sound_once(&self.click_sound);
                    self.request_startup_back(StartupPendingBack::ToAskPlayerName);
                    return;
                }
                if is_key_pressed(KeyCode::Enter) {
                    play_sound_once(&self.click_sound);
                    if self.startup_menu_continue == 0 {
                        if let Some(ns) = self.startup_continue_next_stage {
                            self.begin_normal_play_after_startup(ns, false);
                        }
                    } else {
                        self.startup = StartupState::NicknameMustChangeNotice;
                    }
                }
            }
            StartupState::ContinueNoRecordNotice => {
                if is_key_pressed(KeyCode::Enter) {
                    play_sound_once(&self.click_sound);
                    self.startup_player_name_buffer.clear();
                    STARTUP_NAME_INPUT_QUEUE_CLEARED.store(false, Ordering::Relaxed);
                    self.startup = StartupState::AskPlayerName;
                } else if is_key_pressed(KeyCode::Escape) {
                    play_sound_once(&self.click_sound);
                    self.request_startup_back(StartupPendingBack::ToAskNewOrContinue);
                }
            }
            StartupState::NicknameMustChangeNotice => {
                if is_key_pressed(KeyCode::Enter) {
                    play_sound_once(&self.click_sound);
                    self.player_name.clear();
                    self.startup_continue_next_stage = None;
                    self.startup_continue_max_cleared = None;
                    self.startup_player_name_buffer.clear();
                    STARTUP_NAME_INPUT_QUEUE_CLEARED.store(false, Ordering::Relaxed);
                    self.startup = StartupState::AskPlayerName;
                } else if is_key_pressed(KeyCode::Escape) {
                    play_sound_once(&self.click_sound);
                    self.request_startup_back(StartupPendingBack::ToAskContinueFromLog);
                }
            }
            StartupState::AskDevPassword => {
                if is_key_pressed(KeyCode::Escape) {
                    play_sound_once(&self.click_sound);
                    self.request_startup_back(StartupPendingBack::ToAskPlayerRole);
                    return;
                }
                if !STARTUP_DEV_PASSWORD_INPUT_QUEUE_CLEARED.load(Ordering::Relaxed) {
                    for _ in 0..4096 {
                        if get_char_pressed().is_none() {
                            break;
                        }
                    }
                    STARTUP_DEV_PASSWORD_INPUT_QUEUE_CLEARED.store(true, Ordering::Relaxed);
                    return;
                }
                while let Some(ch) = get_char_pressed() {
                    if ch.is_ascii_alphanumeric() && self.password_buffer.len() < 32 {
                        play_sound_once(&self.click_sound);
                        self.password_buffer.push(ch);
                    }
                }
                if is_key_pressed(KeyCode::Backspace) && !self.password_buffer.is_empty() {
                    play_sound_once(&self.click_sound);
                    self.password_buffer.pop();
                }
                if is_key_pressed(KeyCode::Enter) {
                    // Changeable dev password (8+ letters/numbers) for test mode access.
                    play_sound_once(&self.click_sound);
                    if self.password_buffer == DEV_PASSWORD {
                        self.easy_test_map = true;
                        self.test_mask_enabled = false;
                        self.setup_stage(11, 9);
                        self.story = StoryPhase::Playing;
                        self.password_buffer.clear();
                        self.startup = StartupState::Done;
                    } else {
                        self.easy_test_map = false;
                        self.password_buffer.clear();
                        self.startup_player_name_buffer.clear();
                        self.startup_continue_next_stage = None;
                        self.startup_continue_max_cleared = None;
                        self.reset_normal_lobby_geometry();
                        self.startup_menu_run_type = 0;
                        self.startup = StartupState::AskNewOrContinue;
                    }
                }
            }
            StartupState::ViewRecords => {
                let recs = self.progress.load_summaries_newest_first(50);
                const PAGE: usize = 10;
                if recs.is_empty() {
                    self.startup_records_selected = 0;
                    self.startup_records_scroll = 0;
                } else {
                    let max_selected = recs.len() - 1;
                    self.startup_records_selected = self.startup_records_selected.min(max_selected);
                }
                if is_key_pressed(KeyCode::Escape) {
                    play_sound_once(&self.click_sound);
                    self.request_startup_back(StartupPendingBack::ToAskPlayerRoleFromRecords);
                    return;
                }
                if is_key_pressed(KeyCode::Up) && self.startup_records_selected > 0 {
                    play_sound_once(&self.click_sound);
                    self.startup_records_selected -= 1;
                }
                if is_key_pressed(KeyCode::Down) && self.startup_records_selected + 1 < recs.len() {
                    play_sound_once(&self.click_sound);
                    self.startup_records_selected += 1;
                }
                let max_scroll = recs.len().saturating_sub(PAGE);
                if self.startup_records_selected < self.startup_records_scroll {
                    self.startup_records_scroll = self.startup_records_selected;
                } else if self.startup_records_selected >= self.startup_records_scroll + PAGE {
                    self.startup_records_scroll = self.startup_records_selected + 1 - PAGE;
                }
                self.startup_records_scroll = self.startup_records_scroll.min(max_scroll);

                if is_key_pressed(KeyCode::Enter) && !recs.is_empty() {
                    play_sound_once(&self.click_sound);
                    if let Some(record) = self
                        .progress
                        .load_full_record_by_newest_index(self.startup_records_selected)
                    {
                        if self.start_replay_from_record(record) {
                            self.startup = StartupState::Done;
                        }
                    }
                }
            }
            StartupState::Done => {}
        }
    }

    fn draw_test_mode_shortcuts(&self) {
        let x = 8.0;
        let y = 62.0;
        let w = 620.0;
        let h = 114.0;
        draw_rectangle(x, y, w, h, Color::from_rgba(8, 10, 20, 170));
        draw_rectangle_lines(x, y, w, h, 1.0, Color::from_rgba(130, 150, 220, 180));
        draw_text("Test/Debug Easy Map Shortcuts", x + 10.0, y + 24.0, 22.0, WHITE);
        draw_text(
            "F4: toggle visibility mask | F1: debug overlay | F2: credits",
            x + 10.0,
            y + 52.0,
            19.0,
            Color::from_rgba(180, 210, 255, 255),
        );
        draw_text(
            "WASD/Arrows: move | After win: ↑↓ + Enter on congratulations menu",
            x + 10.0,
            y + 78.0,
            19.0,
            Color::from_rgba(180, 210, 255, 255),
        );
        draw_text(
            "Collect hint items to reveal full map for 5s",
            x + 10.0,
            y + 102.0,
            18.0,
            Color::from_rgba(150, 210, 180, 255),
        );
    }
}

fn format_cell(c: (usize, usize)) -> String {
    format!("({},{})", c.0, c.1)
}

fn credits_toggle_pressed() -> bool {
    is_key_pressed(KeyCode::F2)
}

fn build_hint_cells(cols: usize, rows: usize) -> Vec<(usize, usize)> {
    let mid_x = cols / 2;
    let mid_y = rows / 2;
    let candidates = [
        (0, 0),
        (mid_x, 0),
        (cols.saturating_sub(1), 0),
        (0, mid_y),
        (cols.saturating_sub(1), mid_y),
        (0, rows.saturating_sub(1)),
        (mid_x, rows.saturating_sub(1)),
        (cols.saturating_sub(1), rows.saturating_sub(1)),
    ];
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for c in candidates {
        if c.0 < cols && c.1 < rows && seen.insert(c) {
            out.push(c);
        }
    }
    out
}
