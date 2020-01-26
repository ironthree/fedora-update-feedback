use std::process::Command;

use bodhi::FedoraRelease;

use super::{parse_filename, NVR};

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
