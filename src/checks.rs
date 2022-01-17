use std::collections::HashMap;

use bodhi::{BodhiService, FedoraRelease, Update};

use crate::config::FedoraConfig;
use crate::nvr::NVR;
use crate::query::{query_obsoleted, query_unpushed};
use crate::Command;

pub fn do_check_pending(args: &Command, config: Option<&FedoraConfig>) -> bool {
    args.check_pending || {
        if let Some(config) = config {
            if let Some(cfg) = &config.fuf {
                if let Some(b) = cfg.check_pending {
                    b
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }
}

pub fn do_check_obsoletes(args: &Command, config: Option<&FedoraConfig>) -> bool {
    args.check_obsoleted || {
        if let Some(config) = config {
            if let Some(cfg) = &config.fuf {
                if let Some(b) = cfg.check_obsoleted {
                    b
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }
}

pub fn do_check_unpushed(args: &Command, config: Option<&FedoraConfig>) -> bool {
    args.check_unpushed || {
        if let Some(config) = config {
            if let Some(cfg) = &config.fuf {
                if let Some(b) = cfg.check_unpushed {
                    b
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }
}

fn filter_installed_updates<'a>(
    check_updates: &'a [Update],
    installed_packages: &[NVR],
    builds_for_update: &mut HashMap<String, Vec<String>>,
) -> Result<Vec<&'a Update>, String> {
    let mut installed_matched: Vec<&Update> = Vec::new();

    for update in check_updates {
        let nvrs = update
            .builds
            .iter()
            .map(|b| b.nvr.parse())
            .collect::<Result<Vec<NVR>, String>>()?;

        for nvr in nvrs {
            if installed_packages.contains(&nvr) {
                installed_matched.push(update);

                builds_for_update
                    .entry(update.alias.clone())
                    .and_modify(|e| e.push(nvr.to_string()))
                    .or_insert_with(|| vec![nvr.to_string()]);
            };
        }
    }

    Ok(installed_matched)
}

fn print_update_builds(
    updates: &[&Update],
    src_bin_map: &HashMap<String, Vec<String>>,
    builds_for_update: &mut HashMap<String, Vec<String>>,
) {
    for update in updates {
        println!(" - {}:", update.title);
        // this unwrap is safe since we definitely inserted a value for every update
        for build in builds_for_update.get(update.alias.as_str()).unwrap() {
            let mut binaries: Vec<&str> = Vec::new();
            if let Some(list) = src_bin_map.get(build) {
                binaries.extend(list.iter().map(|s| s.as_str()));
            };

            for binary in binaries {
                println!("   - {}", binary);
            }
        }
    }
}

pub async fn obsoleted_check(
    bodhi: &BodhiService,
    release: &FedoraRelease,
    installed_packages: &[NVR],
    src_bin_map: &HashMap<String, Vec<String>>,
    builds_for_update: &mut HashMap<String, Vec<String>>,
) -> Result<(), String> {
    // get updates in "unpushed" state
    let obsoleted_updates = query_obsoleted(bodhi, release).await?;
    println!();

    let installed_obsoleted = filter_installed_updates(&obsoleted_updates, installed_packages, builds_for_update)?;

    if !installed_obsoleted.is_empty() {
        println!("There are obsoleted updates installed on this system.");
        println!("This probably means your system is not up-to-date.");

        print_update_builds(&installed_obsoleted, src_bin_map, builds_for_update);
    };

    Ok(())
}

pub async fn unpushed_check(
    bodhi: &BodhiService,
    release: &FedoraRelease,
    installed_packages: &[NVR],
    src_bin_map: &HashMap<String, Vec<String>>,
    builds_for_update: &mut HashMap<String, Vec<String>>,
) -> Result<(), String> {
    // get updates in "unpushed" state
    let unpushed_updates = query_unpushed(bodhi, release).await?;
    println!();

    let installed_unpushed = filter_installed_updates(&unpushed_updates, installed_packages, builds_for_update)?;

    if !installed_unpushed.is_empty() {
        println!("There are unpushed updates installed on this system.");
        println!("It is recommended to run 'dnf distro-sync' to clean this up.");

        print_update_builds(&installed_unpushed, src_bin_map, builds_for_update);
    };

    Ok(())
}
