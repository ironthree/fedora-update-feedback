use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;
use tokio::fs::write;

const CACHE_ERROR: &str = "Failed to get cache directory.";
const FILE_NAME: &str = "fedora-update-feedback.ignored";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct IgnoreLists {
    pub ignored_updates: Vec<String>,
    pub ignored_packages: Vec<String>,
}

fn get_ignore_path() -> PathBuf {
    let cache_dir = dirs::cache_dir().expect(CACHE_ERROR);
    cache_dir.join(FILE_NAME)
}

/// Helper function to get list of ignored updates from the cache file.
pub async fn get_ignored() -> Result<IgnoreLists, String> {
    let ignore_path = get_ignore_path();

    let string = read_to_string(ignore_path).await.map_err(|error| error.to_string())?;

    // attempt to parse new JSON format
    let contents: IgnoreLists = match serde_json::from_str(&string) {
        Ok(json) => json,
        Err(_) => IgnoreLists {
            // fall back to line-based list of ignored updates from fuf < 2.0.0
            ignored_packages: Vec::new(),
            ignored_updates: string.trim().split('\n').map(|i| i.to_string()).collect(),
        },
    };

    Ok(contents)
}

/// Helper function to write the list of ignored updates to the cache file.
pub async fn set_ignored(ignored: &IgnoreLists) -> Result<(), String> {
    let ignore_path = get_ignore_path();

    let contents = serde_json::to_string_pretty(ignored).map_err(|error| error.to_string())?;
    write(ignore_path, contents).await.map_err(|error| error.to_string())?;

    Ok(())
}
