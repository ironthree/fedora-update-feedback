use serde::Deserialize;
use tokio::fs::read_to_string;

/// This struct represents the contents of a `~/.config/fedora.toml` file.
/// It includes a mandatory `[FAS]` section, and optional sections for tools.
/// It should look something like this:
///
/// ```toml
/// [FAS]
/// username = USERNAME
///
/// [fedora-update-feedback]
/// check-obsoleted = false
/// check-pending = true
/// check-unpushed = true
/// ```
#[derive(Debug, Deserialize)]
pub struct FedoraConfig {
    /// This section contains information about the user's FAS account.
    #[serde(rename(deserialize = "FAS"))]
    pub fas: FASConfig,
    /// This section contains configuration for fedora-update-feedback.
    #[serde(rename(deserialize = "fedora-update-feedback"))]
    pub fuf: Option<FUFConfig>,
}

/// This config file section contains information about the current user's FAS account.
#[derive(Debug, Deserialize)]
pub struct FASConfig {
    /// User name in the fedora accounts system (FAS)
    pub username: String,
}

/// This config file section contains settings for fedora-update-feedback.
#[derive(Debug, Deserialize)]
pub struct FUFConfig {
    /// Check for installed obsolete updates
    #[serde(rename = "check-obsoleted")]
    pub check_obsoleted: Option<bool>,
    /// Check for installed pending updates
    #[serde(rename = "check-pending")]
    pub check_pending: Option<bool>,
    /// Check for installed unpushed updates
    #[serde(rename = "check-unpushed")]
    pub check_unpushed: Option<bool>,
    /// Save password in session keyring
    #[serde(rename = "save-password")]
    pub save_password: Option<bool>,
}

/// This helper function reads and parses the configuration file.
pub async fn get_config() -> Result<FedoraConfig, String> {
    let home = match dirs::home_dir() {
        Some(path) => path,
        None => {
            return Err(String::from("Unable to determine $HOME."));
        },
    };

    let config_path = home.join(".config/fedora.toml");

    let config_str = match read_to_string(&config_path).await {
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
pub async fn get_legacy_username() -> Result<Option<String>, String> {
    let home = match dirs::home_dir() {
        Some(path) => path,
        None => {
            return Err(String::from("Unable to determine $HOME."));
        },
    };

    let file_path = home.join(".fedora.upn");

    let username = match read_to_string(&file_path).await {
        Ok(string) => Some(string.trim().to_string()),
        Err(error) => {
            return if error.kind() == std::io::ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(error.to_string())
            };
        },
    };

    Ok(username)
}
