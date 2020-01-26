use std::fs::read_to_string;
use std::fs::write;
use std::path::PathBuf;

const CACHE_ERROR: &str = "Failed to get cache directory.";
const FILE_NAME: &str = "fedora-update-feedback.ignored";

fn get_ignore_path() -> PathBuf {
    let cache_dir = dirs::cache_dir().expect(CACHE_ERROR);
    cache_dir.join(FILE_NAME)
}

/// Helper function to get list of ignored updates from the cache file.
pub fn get_ignored() -> Result<Vec<String>, String> {
    let ignore_path = get_ignore_path();

    let ignored = match read_to_string(ignore_path) {
        Ok(contents) => contents.trim().split('\n').map(|i| i.to_string()).collect(),
        Err(error) => return Err(error.to_string()),
    };

    Ok(ignored)
}

/// Helper function to write the list of ignored updates to the cache file.
pub fn set_ignored(ignored: &[String]) -> Result<(), String> {
    let ignore_path = get_ignore_path();

    let contents = ignored.join("\n");

    if let Err(error) = write(ignore_path, contents) {
        return Err(error.to_string());
    };

    Ok(())
}
