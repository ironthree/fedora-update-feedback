use std::collections::HashMap;
use std::process::Command;

use bodhi::FedoraRelease;
use chrono::{DateTime, TimeZone, Utc};

use crate::nvr::NVR;
use crate::parse::parse_filename;

/// This helper function queries RPM for the value of `%{fedora}` on the current system.
pub fn get_release() -> Result<FedoraRelease, String> {
    let output = match Command::new("rpm").arg("--eval").arg("%{fedora}").output() {
        Ok(output) => output,
        Err(error) => {
            return Err(format!("{}", error));
        },
    };

    match output.status.code() {
        Some(x) if x != 0 => {
            return Err(String::from("Failed to run rpm."));
        },
        Some(_) => {},
        None => {
            return Err(String::from("Failed to run rpm."));
        },
    };

    let release_num = match std::str::from_utf8(&output.stdout) {
        Ok(result) => result,
        Err(error) => {
            return Err(format!("{}", error));
        },
    }
    .trim();

    let release = format!("F{}", release_num);

    let release: FedoraRelease = match release.parse() {
        Ok(release) => release,
        Err(error) => return Err(error.to_string()),
    };

    Ok(release)
}

/// This helper function queries `dnf` for the source package names of all currently installed
/// packages.
pub fn get_installed() -> Result<Vec<NVR>, String> {
    // query dnf for installed packages
    let output = match Command::new("dnf")
        .arg("--quiet")
        .arg("repoquery")
        .arg("--cacheonly")
        .arg("--installed")
        .arg("--source")
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            return Err(format!("{}", error));
        },
    };

    match output.status.code() {
        Some(x) if x != 0 => {
            return Err(String::from("Failed to query dnf."));
        },
        Some(_) => {},
        None => {
            return Err(String::from("Failed to query dnf."));
        },
    };

    let installed = match std::str::from_utf8(&output.stdout) {
        Ok(result) => result,
        Err(error) => {
            return Err(format!("{}", error));
        },
    };

    let lines: Vec<&str> = installed.trim().split('\n').collect();

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
pub fn get_summaries() -> Result<HashMap<String, String>, String> {
    // query dnf for installed packages and their summaries
    let output = match Command::new("dnf")
        .arg("--quiet")
        .arg("repoquery")
        .arg("--cacheonly")
        .arg("--installed")
        .arg("--qf")
        .arg("%{name}\t%{summary}")
        .output()
    {
        Ok(output) => output,
        Err(error) => return Err(error.to_string()),
    };

    match output.status.code() {
        Some(x) if x != 0 => return Err(String::from("Failed to query dnf.")),
        Some(_) => {},
        None => return Err(String::from("Failed to query dnf.")),
    }

    let results = match std::str::from_utf8(&output.stdout) {
        Ok(result) => result,
        Err(error) => return Err(error.to_string()),
    };

    let lines: Vec<&str> = results.trim().split('\n').collect();

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
pub fn get_src_bin_map() -> Result<HashMap<String, Vec<String>>, String> {
    // query dnf for installed binary packages and their corresponding source package
    let output = match Command::new("dnf")
        .arg("--quiet")
        .arg("repoquery")
        .arg("--cacheonly")
        .arg("--installed")
        .arg("--qf")
        .arg("%{source_name}-%{version}-%{release} %{name}-%{version}-%{release}.%{arch}")
        .output()
    {
        Ok(output) => output,
        Err(error) => return Err(error.to_string()),
    };

    match output.status.code() {
        Some(x) if x != 0 => return Err(String::from("Failed to query dnf.")),
        Some(_) => {},
        None => return Err(String::from("Failed to query dnf.")),
    }

    let results = match std::str::from_utf8(&output.stdout) {
        Ok(result) => result,
        Err(error) => return Err(format!("{}", error)),
    };

    let lines: Vec<&str> = results.trim().split('\n').collect();

    let mut pkg_map: HashMap<String, Vec<String>> = HashMap::new();

    for line in lines {
        let parts: Vec<&str> = line.split(' ').collect();

        if parts.len() != 2 {
            return Err(String::from("Failed to parse dnf output."));
        };

        let source = parts.get(0).unwrap();
        let binary = parts.get(1).unwrap();

        pkg_map
            .entry((*source).to_string())
            .and_modify(|v| v.push((*binary).to_string()))
            .or_insert_with(|| vec![(*binary).to_string()]);
    }

    Ok(pkg_map)
}

/// This helper function returns a map from binary packages to their installation times.
pub fn get_installation_times() -> Result<HashMap<String, DateTime<Utc>>, String> {
    // query dnf for installed binary packages and their corresponding installation dates
    let output = match Command::new("dnf")
        .arg("--quiet")
        .arg("repoquery")
        .arg("--cacheonly")
        .arg("--installed")
        .arg("--qf")
        .arg("%{name}-%{version}-%{release}.%{arch}\t%{INSTALLTIME}")
        .output()
    {
        Ok(output) => output,
        Err(error) => return Err(format!("{}", error)),
    };

    match output.status.code() {
        Some(x) if x != 0 => return Err(String::from("Failed to query dnf.")),
        Some(_) => {},
        None => return Err(String::from("Failed to query dnf.")),
    }

    let results = match std::str::from_utf8(&output.stdout) {
        Ok(result) => result,
        Err(error) => return Err(format!("{}", error)),
    };

    let lines: Vec<&str> = results.trim().split('\n').collect();

    let mut pkg_map: HashMap<String, DateTime<Utc>> = HashMap::new();

    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();

        if parts.len() != 2 {
            return Err(format!("Failed to parse dnf output: {}", line));
        };

        let binary = parts.get(0).unwrap();
        let installtime = parts.get(1).unwrap();

        let datetime = match Utc.datetime_from_str(installtime, "%Y-%m-%d %H:%M") {
            Ok(datetime) => datetime,
            Err(error) => return Err(format!("Failed to parse dnf output: {}", error)),
        };

        pkg_map.entry((*binary).to_string()).or_insert(datetime);
    }

    Ok(pkg_map)
}
