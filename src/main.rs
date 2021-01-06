#![warn(missing_docs)]
#![warn(clippy::unwrap_used)]

//! This crate contains the `fedora-update-feedback` binary and some helper functionality. If
//! something turns out to be generally useful, it can be upstreamed into either the
//! [`fedora`][fedora-rs] or [`bodhi`][bodhi] crates.
//!
//! [fedora-rs]: https://crates.io/crates/fedora
//! [bodhi-rs]: https://crates.io/crates/bodhi

use std::collections::HashMap;

use bodhi::error::QueryError;
use bodhi::*;

use structopt::StructOpt;

mod config;
mod ignore;
mod input;
mod nvr;
mod output;
mod parse;
mod query;
mod sysinfo;

use config::{get_config, get_legacy_username};
use ignore::{get_ignored, set_ignored};
use input::{ask_feedback, Feedback};
use nvr::NVR;
use parse::parse_nvr;
use query::{query_obsoleted, query_pending, query_testing, query_unpushed};
use sysinfo::{get_installation_times, get_installed, get_release, get_src_bin_map, get_summaries};

/// There are some features that are configurable with the config file located at
/// ~/.config/fedora.toml.
///
/// The [fedora-update-feedback] section can contain values for:
///
/// check-obsoleted: Corresponds to the --check-obsoleted CLI switch - additionally
/// checks whether obsoleted updates are installed on the system.
///
/// check-pending: Corresponds to the --check-pending CLI switch - additionally
/// queries bodhi for updates that are still pending.
///
/// check-unpushed: Corresponds to the --check-unpushed CLI switch - additionally
/// checks whether unpushed updates are installed on the system.
///
/// save-password: Try to saves the FAS password in the session keyring. To ignore
/// a password that was stored in the session keyring (for example, if you changed
/// it, or made a typo when it was prompted), use the --ignore-keyring CLI switch
/// to ask for the password again.
#[derive(Debug, StructOpt)]
struct Command {
    /// Override or provide FAS username
    #[structopt(long, short)]
    username: Option<String>,
    /// Check for installed obsolete updates
    #[structopt(long, short = "O")]
    check_obsoleted: bool,
    /// Include updates in "pending" state
    #[structopt(long, short = "P")]
    check_pending: bool,
    /// Check for installed unpushed updates
    #[structopt(long, short = "U")]
    check_unpushed: bool,
    /// Clear ignored updates
    #[structopt(long, short = "i")]
    clear_ignored: bool,
    /// Ignore password stored in session keyring
    #[structopt(long)]
    ignore_keyring: bool,
}

/// This function prompts the user for their FAS password.
fn read_password() -> String {
    rpassword::prompt_password_stdout("FAS Password: ").expect("Failed to read password from stdin.")
}

/// This function asks for and stores the password in the session keyring.
fn get_store_password(clear: bool) -> Result<String, String> {
    let ss = match secret_service::SecretService::new(secret_service::EncryptionType::Dh) {
        Ok(ss) => ss,
        Err(error) => {
            println!("Failed to initialize SecretService client: {}", error);
            return Ok(read_password());
        },
    };

    let collection = match ss.get_default_collection() {
        Ok(c) => c,
        Err(error) => {
            println!("Failed to query SecretService: {}", error);
            return Ok(read_password());
        },
    };

    let mut attributes = HashMap::new();
    attributes.insert("fedora-update-feedback", "FAS Password");

    let store = |password: &str, replace: bool| {
        if let Err(error) = collection.create_item(
            "fedora-update-feedback",
            attributes.clone(),
            &password.as_bytes(),
            replace,
            "password",
        ) {
            println!("Failed to save password with SecretService: {}", error);
        }
    };

    let items = match collection.search_items(attributes.clone()) {
        Ok(items) => items,
        Err(error) => {
            format!("Failed to query SecretService: {}", error);
            return Ok(read_password());
        },
    };

    if clear {
        let password = read_password();
        store(&password, true);
        return Ok(password);
    };

    let password = match items.get(0) {
        Some(item) => match item.get_secret() {
            Ok(secret) => match String::from_utf8(secret) {
                Ok(valid) => valid,
                Err(error) => {
                    println!("Stored password was not valid UTF-8: {}", error);

                    let password = read_password();
                    store(&password, true);

                    password
                },
            },
            Err(error) => {
                println!("Password was not stored correctly: {}", error);

                let password = read_password();
                store(&password, true);

                password
            },
        },
        None => {
            let password = read_password();
            store(&password, false);

            password
        },
    };

    Ok(password)
}

#[allow(clippy::cognitive_complexity)]
fn main() -> Result<(), String> {
    let args: Command = Command::from_args();

    let config = if let Ok(config) = get_config() {
        Some(config)
    } else {
        None
    };

    let username = if let Some(username) = &args.username {
        username.clone()
    } else if let Some(config) = &config {
        config.fas.username.clone()
    } else if let Ok(Some(username)) = get_legacy_username() {
        username
    } else {
        return Err(String::from("Failed to read ~/.config/fedora.toml and ~/.fedora.upn."));
    };

    println!("Username: {}", &username);

    // read password from libsecret-1 or fall back to command line prompt
    let password = match &config {
        Some(config) => match &config.fuf {
            Some(fuf) => match fuf.save_password {
                Some(x) if x => get_store_password(args.ignore_keyring)?,
                _ => read_password(),
            },
            None => read_password(),
        },
        None => read_password(),
    };

    // query bodhi for packages in updates-testing
    println!("Authenticating with bodhi ...");
    let bodhi = match BodhiServiceBuilder::default()
        .authentication(&username, &password)
        .build()
    {
        Ok(bodhi) => bodhi,
        Err(error) => {
            return Err(format!("{}", error));
        },
    };

    // query rpm for current release
    let release = get_release()?;

    // query DNF for installed packages
    println!("Querying dnf for installed packages ...");
    let installed_packages = get_installed()?;
    // query DNF for source -> binary package map
    let src_bin_map = get_src_bin_map()?;

    println!("Querying bodhi for updates ...");
    let mut updates: Vec<Update> = Vec::new();

    // get updates in "testing" state
    let testing_updates = query_testing(&bodhi, release)?;
    updates.extend(testing_updates);
    println!();

    let check_pending = args.check_pending || {
        if let Some(config) = &config {
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
    };

    if check_pending {
        // get updates in "pending" state
        let pending_updates = query_pending(&bodhi, release)?;
        updates.extend(pending_updates);
        println!();
    };

    // filter out updates created by the current user
    let updates: Vec<Update> = updates
        .into_iter()
        .filter(|update| update.user.name != username)
        .collect();

    // filter out updates that were already commented on
    let mut relevant_updates: Vec<&Update> = Vec::new();
    for update in &updates {
        if let Some(comments) = &update.comments {
            let mut commented = false;

            for comment in comments {
                if comment.user.name == username {
                    commented = true;
                };
            }

            if !commented {
                relevant_updates.push(update);
            };
        } else {
            relevant_updates.push(update);
        };
    }

    // filter out updates for packages that are not installed;
    // and remember which builds are installed for which update
    let mut installed_updates: Vec<&Update> = Vec::new();
    let mut builds_for_update: HashMap<String, Vec<String>> = HashMap::new();

    for update in &relevant_updates {
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
                installed_updates.push(&update);

                builds_for_update
                    .entry(update.alias.clone())
                    .and_modify(|e| e.push(nvr.to_string()))
                    .or_insert_with(|| vec![nvr.to_string()]);
            };
        }
    }

    if installed_updates.is_empty() {
        return Ok(());
    };

    // deduplicate updates with multiple builds
    installed_updates.sort_by(|a, b| a.alias.cmp(&b.alias));
    installed_updates.dedup_by(|a, b| a.alias == b.alias);

    // sort updates by submission date
    installed_updates.sort_by(|a, b| a.date_submitted.cmp(&b.date_submitted));

    let mut rl = rustyline::Editor::<()>::new();

    let mut ignored = if !args.clear_ignored {
        match get_ignored() {
            Ok(ignored) => ignored,
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };

    // remove old updates from ignored list
    ignored.retain(|i| installed_updates.iter().map(|u| &u.alias).any(|x| x == i));

    // query dnf for package summaries
    let summaries = get_summaries()?;

    // query dnf for when the updates were installed
    let install_times = get_installation_times()?;

    for update in installed_updates {
        if ignored.contains(&update.alias) {
            println!("Skipping ignored update: {}", &update.alias);
            continue;
        };

        // this unwrap is safe since we definitely inserted a value for every update
        let builds = builds_for_update.get(update.alias.as_str()).unwrap();

        let mut binaries: Vec<&str> = Vec::new();
        for build in builds {
            if let Some(list) = src_bin_map.get(build) {
                binaries.extend(list.iter().map(|s| s.as_str()));
            };
        }

        let feedback = ask_feedback(&mut rl, update, &binaries, &summaries, &install_times)?;

        match feedback {
            Feedback::Cancel => {
                println!("Cancelling.");
                break;
            },
            Feedback::Ignore => {
                println!("Ignoring.");
                ignored.push(update.alias.clone());
                continue;
            },
            Feedback::Skip => {
                println!("Skipping.");
                continue;
            },
            Feedback::Values {
                comment,
                karma,
                bug_feedback,
                testcase_feedback,
            } => {
                if let (None, Karma::Neutral) = (&comment, karma) {
                    println!("Provided neither a comment nor karma feedback, skipping update.");
                    continue;
                };

                let mut builder = CommentBuilder::new(&update.alias).karma(karma);

                if let Some(text) = &comment {
                    builder = builder.text(text);
                };

                for (id, karma) in bug_feedback {
                    builder = builder.bug_feedback(id, karma);
                }

                for (name, karma) in testcase_feedback {
                    builder = builder.testcase_feedback(name, karma);
                }

                let new_comment: Result<NewComment, QueryError> = bodhi.create(&builder);

                match new_comment {
                    Ok(value) => {
                        println!("Comment created.");

                        if !value.caveats.is_empty() {
                            println!("Server messages:");

                            for caveat in &value.caveats {
                                println!("- {:?}", caveat);
                            }
                        }
                    },
                    Err(error) => {
                        println!("{}", error);
                        continue;
                    },
                };
            },
        };
    }

    let check_obsoleted = args.check_obsoleted || {
        if let Some(config) = &config {
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
    };

    if check_obsoleted {
        // get updates in "unpushed" state
        let obsoleted_updates = query_obsoleted(&bodhi, release)?;
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
                    installed_obsoleted.push(&update);

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
    };

    let check_unpushed = args.check_unpushed || {
        if let Some(config) = &config {
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
    };

    if check_unpushed {
        // get updates in "unpushed" state
        let unpushed_updates = query_unpushed(&bodhi, release)?;
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
                    installed_unpushed.push(&update);

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
    };

    if let Err(error) = set_ignored(&ignored) {
        println!("Failed to write ignored updates to disk.");
        println!("{}", error);
    };

    Ok(())
}
