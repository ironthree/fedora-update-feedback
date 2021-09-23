use std::collections::HashMap;

use bodhi::{BodhiService, FedoraRelease, Update};

use crate::config::FedoraConfig;
use crate::nvr::NVR;
use crate::parse::parse_nvr;
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

pub fn obsoleted_check(
    bodhi: &BodhiService,
    release: FedoraRelease,
    installed_packages: &[NVR],
    src_bin_map: &HashMap<String, Vec<String>>,
    builds_for_update: &mut HashMap<String, Vec<String>>,
) -> Result<(), String> {
    // get updates in "unpushed" state
    let obsoleted_updates = query_obsoleted(bodhi, release)?;
    println!();

    let mut installed_obsoleted: Vec<&Update> = Vec::new();
    for update in &obsoleted_updates {
        let mut nvrs: Vec<NVR> = Vec::new();

        for build in &update.builds {
            let (n, v, r) = parse_nvr(&build.nvr)?;
            nvrs.push(NVR {
                n: n.to_string(),
                v: v.to_string(),
                r: r.to_string(),
            });
        }

        for nvr in nvrs {
            if installed_packages.contains(&nvr) {
                installed_obsoleted.push(update);

                builds_for_update
                    .entry(update.alias.clone())
                    .and_modify(|e| e.push(nvr.to_string()))
                    .or_insert_with(|| vec![nvr.to_string()]);
            };
        }
    }

    if !installed_obsoleted.is_empty() {
        println!("There are obsoleted updates installed on this system.");
        println!("This probably means your system is not up-to-date.");

        for update in installed_obsoleted {
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
    };

    Ok(())
}

pub fn unpushed_check(
    bodhi: &BodhiService,
    release: FedoraRelease,
    installed_packages: &[NVR],
    src_bin_map: &HashMap<String, Vec<String>>,
    builds_for_update: &mut HashMap<String, Vec<String>>,
) -> Result<(), String> {
    // get updates in "unpushed" state
    let unpushed_updates = query_unpushed(bodhi, release)?;
    println!();

    let mut installed_unpushed: Vec<&Update> = Vec::new();
    for update in &unpushed_updates {
        let mut nvrs: Vec<NVR> = Vec::new();

        for build in &update.builds {
            let (n, v, r) = parse_nvr(&build.nvr)?;
            nvrs.push(NVR {
                n: n.to_string(),
                v: v.to_string(),
                r: r.to_string(),
            });
        }

        for nvr in nvrs {
            if installed_packages.contains(&nvr) {
                installed_unpushed.push(update);

                builds_for_update
                    .entry(update.alias.clone())
                    .and_modify(|e| e.push(nvr.to_string()))
                    .or_insert_with(|| vec![nvr.to_string()]);
            };
        }
    }

    if !installed_unpushed.is_empty() {
        println!("There are unpushed updates installed on this system.");
        println!("It is recommended to run 'dnf distro-sync' to clean this up.");

        for update in installed_unpushed {
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
    };

    Ok(())
}
