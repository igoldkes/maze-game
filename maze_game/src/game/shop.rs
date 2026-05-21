use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ShopItem {
    todo!(),
}


/// Same JSON object as [`ProgressRecord`] but **omits `cells`**! Serde ignores the `cells` field in the file,
/// so the records menu can load without allocating megabytes of wall data per line.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
pub struct ShopItemListRow {
    todo!(),
}


pub struct ShopItemService {
    todo!(),
}