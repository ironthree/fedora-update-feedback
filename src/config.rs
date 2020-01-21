use std::fs::read_to_string;

use serde::Deserialize;

/// This struct represents the contents of a `~/.config/fedora.toml` file.
#[derive(Debug, Deserialize)]
pub struct FedoraConfig {
    /// This section contains information about the user's FAS account.
    #[serde(rename(deserialize = "FAS"))]
    pub fas: FASConfig,
}

/// This config file section contains information about the current user's FAS account.
#[derive(Debug, Deserialize)]
pub struct FASConfig {
    /// User name in the fedora accounts system (FAS)
    pub username: String,
}

/// This helper function reads and parses the configuration file.
pub fn get_config() -> Result<FedoraConfig, String> {
    let home = match dirs::home_dir() {
        Some(path) => path,
        None => {
            return Err(String::from("Unable to determine $HOME."));
        },
    };

    let config_path = home.join(".config/fedora.toml");

    let config_str = match read_to_string(&config_path) {
        Ok(string) => string,
        Err(_) => {
            return Err(String::from(
                "Unable to read configuration file from ~/.config/fedora.toml",
            ));
        },
    };

    let config: FedoraConfig = match toml::from_str(&config_str) {
        Ok(config) => config,
        Err(_) => {
            return Err(String::from(
                "Unable to parse configuration file from ~/.config/fedora.toml",
            ));
        },
    };

    Ok(config)
}

/// This helper function reads the username from the legacy `~/.fedora.upn` file.
pub fn get_legacy_username() -> Result<Option<String>, String> {
    let home = match dirs::home_dir() {
        Some(path) => path,
        None => {
            return Err(String::from("Unable to determine $HOME."));
        },
    };

    let file_path = home.join(".fedora.upn");

    let username = match read_to_string(&file_path) {
        Ok(string) => Some(string.trim().to_string()),
        Err(error) => {
            return if error.kind() == std::io::ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(error.to_string())
            }
        },
    };

    Ok(username)
}
