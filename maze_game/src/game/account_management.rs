//! Local save lookup by player nickname: detect prior clears and compute the next stage to play.
//!
//! Only **completed** runs are persisted (`append_success` on maze exit + win flow). In-progress
//! stages are not logged, so they never affect “continue” — only the highest **saved** cleared stage.

use super::progress::ProgressService;

/// Highest stage number this nickname has **cleared** (saved as a non-replay record), if any.
pub fn max_cleared_stage_for_player(progress: &ProgressService, player_name: &str) -> Option<usize> {
    let trimmed = player_name.trim();
    if trimmed.is_empty() {
        return None;
    }
    progress
        .load_summaries_newest_first(500)
        .into_iter()
        .filter(|r| !r.is_replay && r.player_name.trim() == trimmed)
        .map(|r| r.stage as usize)
        .max()
}

/// Next stage to play after the best clear (e.g. cleared 5 → play stage 6). `None` if no prior clears.
pub fn next_stage_to_play_after_clears(
    progress: &ProgressService,
    player_name: &str,
) -> Option<usize> {
    max_cleared_stage_for_player(progress, player_name).map(|m| m.saturating_add(1))
}
