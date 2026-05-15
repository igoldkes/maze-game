//! Central gameplay-policy helpers:
//! - when movement is allowed
//! - when camera mask is active
//! - when full map should be visible

use super::story::StoryPhase;
use super::{EndMenuState, GameState};

impl GameState {
    pub(super) fn can_move_now(&self) -> bool {
        // Never allow movement while map overlays are up.
        if self.story.show_paper_overlay() || self.story.show_map_item() || self.hint_map_timer > 0.0 {
            return false;
        }
        let can_move = if self.easy_test_map {
            self.end_menu == EndMenuState::Hidden
        } else {
            self.story.movement_allowed() && self.end_menu == EndMenuState::Hidden
        };
        can_move && !self.show_credits
    }

    pub(super) fn normal_mask_active(&self) -> bool {
        !self.easy_test_map
            && matches!(self.story, StoryPhase::Playing)
            && self.hint_map_timer <= 0.0
            && !self.story.show_map_item()
            && !(self.normal_dev_authenticated && self.normal_dev_f3_full_map)
            && self.end_menu == EndMenuState::Hidden
    }

    pub(super) fn use_camera_mask(&self) -> bool {
        if self.easy_test_map {
            self.test_mask_enabled
        } else {
            self.normal_mask_active()
        }
    }

    pub(super) fn full_map_visible_now(&self) -> bool {
        self.story.show_map_item()
            || self.hint_map_timer > 0.0
            || (!self.easy_test_map && self.normal_dev_authenticated && self.normal_dev_f3_full_map)
            || matches!(self.story, StoryPhase::Won)
    }
}
