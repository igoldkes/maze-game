//! JSON Lines log of cleared stages: layout + generator + **elapsed time** (no wall-clock timestamp).
//! Stored under the OS **local data** directory (e.g. `~/.local/share/maze_game/` on Linux) so each machine keeps its own file.
//! Enough data to **replay the same maze** (`cells` + dimensions + generator); `elapsed_secs` is only a stat.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
/// Per-user / per-machine save path (not the project folder).
pub fn default_progress_file() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("maze_game").join("player_progress.jsonl")
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProgressRecord {
    pub schema_version: u32,
    #[serde(default = "default_record_id")]
    pub record_id: String,
    pub player_name: String,
    pub stage: u32,
    pub maze_w: usize,
    pub maze_h: usize,
    pub start_x: usize,
    pub start_y: usize,
    pub exit_x: usize,
    pub exit_y: usize,
    /// `"prim"` (randomized Prim) or `"dfs"` (recursive backtracking)—matches how this maze was built.
    pub maze_generator: String,
    /// Seconds from first `Playing` moment for this stage until the win save (not a calendar timestamp).
    pub elapsed_secs: f32,
    /// True if this run was started from a previously saved maze replay.
    #[serde(default)]
    pub is_replay: bool,
    /// Source record id for replay runs; `None` for normal runs.
    #[serde(default)]
    pub replay_of_record_id: Option<String>,
    /// Optional baseline time used for replay comparison UI.
    #[serde(default)]
    pub baseline_secs: Option<f32>,
    /// Wall bits per cell, row-major — reconstruct with `Maze::new(maze_w, maze_h, cells)`.
    pub cells: Vec<u8>,
}

/// Same JSON object as [`ProgressRecord`] but **omits `cells`**! Serde ignores the `cells` field in the file,
/// so the records menu can load without allocating megabytes of wall data per line.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
pub struct ProgressListRow {
    pub schema_version: u32,
    #[serde(default = "default_record_id")]
    pub record_id: String,
    pub player_name: String,
    pub stage: u32,
    pub maze_w: usize,
    pub maze_h: usize,
    pub start_x: usize,
    pub start_y: usize,
    pub exit_x: usize,
    pub exit_y: usize,
    pub maze_generator: String,
    pub elapsed_secs: f32,
    #[serde(default)]
    pub is_replay: bool,
    #[serde(default)]
    pub replay_of_record_id: Option<String>,
    #[serde(default)]
    pub baseline_secs: Option<f32>,
}

impl ProgressRecord {
    pub const SCHEMA_V4: u32 = 4;

    #[allow(clippy::too_many_arguments)]
    pub fn new_run_snapshot(
        player_name: String,
        stage: u32,
        maze_w: usize,
        maze_h: usize,
        start: (usize, usize),
        exit: (usize, usize),
        maze_generator: String,
        elapsed_secs: f32,
        cells: Vec<u8>,
    ) -> Self {
        Self {
            schema_version: Self::SCHEMA_V4,
            record_id: next_record_id(),
            player_name,
            stage,
            maze_w,
            maze_h,
            start_x: start.0,
            start_y: start.1,
            exit_x: exit.0,
            exit_y: exit.1,
            maze_generator,
            elapsed_secs,
            is_replay: false,
            replay_of_record_id: None,
            baseline_secs: None,
            cells,
        }
    }
}

fn default_record_id() -> String {
    "legacy".to_string()
}

fn next_record_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("rec-{nanos}")
}

pub struct ProgressService {
    path: PathBuf,
    recent: VecDeque<ProgressRecord>,
    recent_cap: usize,
}

impl ProgressService {
    pub fn new_default() -> Self {
        let path = default_progress_file();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        Self::with_path(path, 32)
    }

    pub fn with_path<P: AsRef<Path>>(path: P, recent_cap: usize) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            recent: VecDeque::new(),
            recent_cap,
        }
    }

    pub fn append_success(&mut self, record: &ProgressRecord) -> std::io::Result<()> {
        let line = serde_json::to_string(record).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("json: {e}"))
        })?;
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(f, "{line}")?;

        if self.recent_cap > 0 {
            if self.recent.len() >= self.recent_cap {
                self.recent.pop_front();
            }
            self.recent.push_back(record.clone());
        }
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// At least one completed maze has been saved (file non-empty).
    pub fn has_saved_records(&self) -> bool {
        std::fs::metadata(&self.path)
            .map(|m| m.len() > 0)
            .unwrap_or(false)
    }

    /// Newest saves first for the **UI list** — skips `cells` in JSON (see [`ProgressListRow`]).
    pub fn load_summaries_newest_first(&self, max: usize) -> Vec<ProgressListRow> {
        self.load_jsonl_newest_first::<ProgressListRow>(max)
    }

    /// Every readable row in file order (oldest first) for aggregation; omits `cells` via [`ProgressListRow`].
    fn load_all_summaries_oldest_first(&self) -> Vec<ProgressListRow> {
        let Ok(contents) = std::fs::read_to_string(&self.path) else {
            return vec![];
        };
        contents
            .lines()
            .filter_map(|line| {
                let t = line.trim();
                if t.is_empty() {
                    return None;
                }
                serde_json::from_str(t).ok()
            })
            .collect()
    }

    /// For each nickname, the highest stage number seen in any saved line (including replays).
    /// Sorted by stage descending, then name ascending. Empty names are skipped.
    pub fn leaderboard_max_stage_per_player(&self) -> Vec<(String, u32)> {
        let rows = self.load_all_summaries_oldest_first();
        let mut best: HashMap<String, u32> = HashMap::new();
        for r in rows {
            let name = r.player_name.trim();
            if name.is_empty() {
                continue;
            }
            let name = name.to_string();
            best.entry(name)
                .and_modify(|m| *m = (*m).max(r.stage))
                .or_insert(r.stage);
        }
        let mut v: Vec<(String, u32)> = best.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase())));
        v
    }

    /// Full record rows (including `cells`) for replay loading.
    pub fn load_full_records_newest_first(&self, max: usize) -> Vec<ProgressRecord> {
        self.load_jsonl_newest_first::<ProgressRecord>(max)
    }

    /// Convenience accessor for records UI selection index.
    pub fn load_full_record_by_newest_index(&self, index: usize) -> Option<ProgressRecord> {
        self.load_full_records_newest_first(index + 1)
            .into_iter()
            .nth(index)
    }

    fn load_jsonl_newest_first<T>(&self, max: usize) -> Vec<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let Ok(contents) = std::fs::read_to_string(&self.path) else {
            return vec![];
        };
        let mut parsed: Vec<T> = contents
            .lines()
            .filter_map(|line| {
                let t = line.trim();
                if t.is_empty() {
                    return None;
                }
                serde_json::from_str(t).ok()
            })
            .collect();
        parsed.reverse();
        if parsed.len() > max {
            parsed.truncate(max);
        }
        parsed
    }

    // pub fn replay(&self) {
    //     let recent = &self.recent;
    // }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_row_skips_cells_payload() {
        let full = ProgressRecord::new_run_snapshot(
            "Bob".into(),
            // "333".to_string(),
            1,
            2,
            2,
            (0, 0),
            (1, 1),
            "dfs".into(),
            9.25,
            vec![0x1, 0x2, 0x3, 0x4],
        );
        let json = serde_json::to_string(&full).unwrap();
        let row: ProgressListRow = serde_json::from_str(&json).unwrap();
        assert_eq!(row.player_name, "Bob");
        assert_eq!(row.elapsed_secs, 9.25);
    }

    #[test]
    fn leaderboard_picks_max_stage_per_player() {
        let path = std::env::temp_dir().join("maze_game_lb_test_only.jsonl");
        let _ = std::fs::remove_file(&path);
        let mut p = ProgressService::with_path(&path, 4);
        let r1 = ProgressRecord::new_run_snapshot(
            "Ada".into(),
            2,
            3,
            3,
            (0, 0),
            (2, 2),
            "prim".into(),
            10.0,
            vec![0xf; 9],
        );
        let r2 = ProgressRecord::new_run_snapshot(
            "Ada".into(),
            5,
            4,
            4,
            (0, 0),
            (3, 3),
            "dfs".into(),
            20.0,
            vec![0xf; 16],
        );
        let r3 = ProgressRecord::new_run_snapshot(
            "Bob".into(),
            1,
            2,
            2,
            (0, 0),
            (1, 1),
            "prim".into(),
            5.0,
            vec![0xf; 4],
        );
        p.append_success(&r1).unwrap();
        p.append_success(&r2).unwrap();
        p.append_success(&r3).unwrap();
        let lb = p.leaderboard_max_stage_per_player();
        assert_eq!(lb.len(), 2);
        assert_eq!(lb[0], ("Ada".to_string(), 5));
        assert_eq!(lb[1], ("Bob".to_string(), 1));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn record_roundtrip_json() {
        let r = ProgressRecord::new_run_snapshot(
            "Ada".into(),
            // "333".to_string(),
            2,
            3,
            3,
            (1, 1),
            (2, 2),
            "prim".into(),
            42.5,
            vec![0xf; 9],
        );
        let s = serde_json::to_string(&r).unwrap();
        let back: ProgressRecord = serde_json::from_str(&s).unwrap();
        assert_eq!(r, back);
        assert!(!s.contains("ts_unix"));
    }
}
