use std::collections::HashMap;

use bodhi::{FedoraRelease, InvalidValueError};
use chrono::{DateTime, NaiveDateTime, Utc};
use tokio::process::Command;

use crate::nvr::NVR;
use crate::parse::parse_filename;

fn handle_status(code: Option<i32>, message: &str) -> Result<(), String> {
    match code {
        Some(x) if x != 0 => Err(String::from(message)),
        Some(_) => Ok(()),
        None => Err(String::from(message)),
    }
}

/// This helper function queries RPM for the value of `%{fedora}` on the current system.
pub async fn get_release() -> Result<FedoraRelease, String> {
    // use RPM to expand the `%{fedora}` macro
    let output = Command::new("rpm")
        .arg("--eval")
        .arg("%{fedora}")
        .output()
        .await
        .map_err(|error| error.to_string())?;

    handle_status(output.status.code(), "Failed to run rpm.")?;

    let release_num = std::str::from_utf8(&output.stdout)
        .map_err(|error| error.to_string())?
        .trim();

    let release = format!("F{}", release_num);

    let release: FedoraRelease = release.parse().map_err(|error: InvalidValueError| error.to_string())?;

    Ok(release)
}

/// This helper function queries `dnf` whether the "updates-testing" repository is enabled.
pub async fn is_update_testing_enabled() -> Result<bool, String> {
    // query dnf for enabled repositories, limiting results to those matching "updates-testing"
    let output = Command::new("dnf")
        .arg("repolist")
        .arg("--enabled")
        .arg("updates-testing")
        .output()
        .await
        .map_err(|error| error.to_string())?;

    handle_status(output.status.code(), "Failed to query dnf.")?;

    // if standard output is empty, the repository is not enabled
    Ok(!output.stdout.is_empty())
}

/// This helper function queries `dnf` for the source package names of all currently installed
/// packages.
pub async fn get_installed() -> Result<Vec<NVR>, String> {
    // query dnf for installed packages
    let output = Command::new("dnf")
        .arg("--quiet")
        .arg("repoquery")
        .arg("--cacheonly")
        .arg("--installed")
        .arg("--source")
        .output()
        .await
        .map_err(|error| error.to_string())?;

    handle_status(output.status.code(), "Failed to query dnf.")?;

    let installed = std::str::from_utf8(&output.stdout).map_err(|error| error.to_string())?;
    let lines = installed.trim().lines();

    let mut packages: Vec<NVR> = Vec::new();
    for line in lines {
        let (n, _, v, r, _) = parse_filename(line)?;
        packages.push(NVR {
            n: n.to_string(),
            v: v.to_string(),
            r: r.to_string(),
        });
    }

    Ok(packages)
}

/// This helper function queries `dnf` for the `Summary` header of installed packages.
pub async fn get_summaries() -> Result<HashMap<String, String>, String> {
    // query dnf for installed packages and their summaries
    let output = Command::new("dnf")
        .arg("--quiet")
        .arg("repoquery")
        .arg("--cacheonly")
        .arg("--installed")
        .arg("--qf")
        .arg("%{name}\t%{summary}")
        .output()
        .await
        .map_err(|error| error.to_string())?;

    handle_status(output.status.code(), "Failed to query dnf.")?;

    let results = std::str::from_utf8(&output.stdout).map_err(|error| error.to_string())?;
    let lines = results.trim().lines();

    let mut summaries: HashMap<String, String> = HashMap::new();
    for line in lines {
        let mut split = line.split('\t');
        match (split.next(), split.next(), split.next()) {
            (Some(name), Some(summary), None) => {
                summaries.insert(name.to_string(), summary.to_string());
            },
            _ => return Err(format!("Failed to parse: {}", line)),
        }
    }

    Ok(summaries)
}

/// This helper function returns a map from source -> binary package NVRs for installed packages.
pub async fn get_src_bin_map() -> Result<HashMap<String, Vec<String>>, String> {
    // query dnf for installed binary packages and their corresponding source package
    let output = Command::new("dnf")
        .arg("--quiet")
        .arg("repoquery")
        .arg("--cacheonly")
        .arg("--installed")
        .arg("--qf")
        .arg("%{source_name}-%{version}-%{release} %{name}-%{version}-%{release}.%{arch}")
        .output()
        .await
        .map_err(|error| error.to_string())?;

    handle_status(output.status.code(), "Failed to query dnf.")?;

    let results = std::str::from_utf8(&output.stdout).map_err(|error| error.to_string())?;
    let lines: Vec<&str> = results.trim().split('\n').collect();

    let mut pkg_map: HashMap<String, Vec<String>> = HashMap::new();
    for line in lines {
        let mut parts = line.split(' ');

        let (source, binary) = match (parts.next(), parts.next(), parts.next()) {
            (Some(source), Some(binary), None) => (source, binary),
            _ => return Err(String::from("Failed to parse dnf output.")),
        };

        pkg_map
            .entry((*source).to_string())
            .and_modify(|v| v.push((*binary).to_string()))
            .or_insert_with(|| vec![(*binary).to_string()]);
    }

    Ok(pkg_map)
}

/// This helper function returns a map from binary packages to their installation times.
pub async fn get_installation_times() -> Result<HashMap<String, DateTime<Utc>>, String> {
    // query dnf for installed binary packages and their corresponding installation dates
    let output = Command::new("dnf")
        .arg("--quiet")
        .arg("repoquery")
        .arg("--cacheonly")
        .arg("--installed")
        .arg("--qf")
        .arg("%{name}-%{version}-%{release}.%{arch}\t%{installtime}")
        .output()
        .await
        .map_err(|error| error.to_string())?;

    handle_status(output.status.code(), "Failed to query dnf.")?;

    let results = std::str::from_utf8(&output.stdout).map_err(|error| error.to_string())?;
    let lines: Vec<&str> = results.trim().split('\n').collect();

    let mut pkg_map: HashMap<String, DateTime<Utc>> = HashMap::new();
    for line in lines {
        let mut parts = line.split('\t');

        let (binary, installtime) = match (parts.next(), parts.next(), parts.next()) {
            (Some(binary), Some(installtime), None) => (binary, installtime),
            _ => return Err(format!("Failed to parse dnf output: {}", line)),
        };

        let datetime = match NaiveDateTime::parse_from_str(installtime, "%Y-%m-%d %H:%M") {
            Ok(datetime) => datetime.and_utc(),
            Err(error) => return Err(format!("Failed to parse dnf output: {}", error)),
        };

        pkg_map.entry((*binary).to_string()).or_insert(datetime);
    }

    Ok(pkg_map)
}
