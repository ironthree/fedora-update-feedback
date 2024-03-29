#![warn(missing_docs)]
#![warn(clippy::unwrap_used)]

//! This crate contains the `fedora-update-feedback` binary and some helper functionality. If
//! something turns out to be generally useful, it can be upstreamed into either the
//! [`fedora`][fedora] or [`bodhi`][bodhi] crates.
//!
//! [fedora-rs]: https://crates.io/crates/fedora
//! [bodhi-rs]: https://crates.io/crates/bodhi

use std::collections::HashMap;

use bodhi::error::QueryError;
use bodhi::{BodhiClientBuilder, BugFeedbackData, CommentCreator, Karma, NewComment, TestCaseFeedbackData, Update};
use clap::Parser;

mod checks;
mod cli;
mod config;
mod ignore;
mod input;
mod nvr;
mod output;
mod parse;
mod query;
mod secrets;
mod sysinfo;

use checks::{do_check_obsoletes, do_check_pending, do_check_unpushed, obsoleted_check, unpushed_check};
use cli::Command;
use config::{get_config, get_legacy_username};
use ignore::{get_ignored, set_ignored, IgnoreLists};
use input::{ask_feedback, Feedback, Progress};
use nvr::NVR;
use output::print_server_messages;
use query::{query_pending, query_testing};
use secrets::{get_store_password, read_password};
use sysinfo::{
    get_installation_times,
    get_installed,
    get_release,
    get_src_bin_map,
    get_summaries,
    is_update_testing_enabled,
};

const USER_AGENT: &str = concat!("fedora-update-feedback v", env!("CARGO_PKG_VERSION"));

fn has_already_commented(update: &Update, user: &str) -> (bool, bool) {
    if let Some(comments) = update.comments.as_ref() {
        let mut already_commented = false;
        let mut reset = false;

        comments.iter().for_each(|comment| {
            // user has commented, so karma reset either never happened or happened before the comment
            if comment.user.name == user && comment.karma != Karma::Neutral {
                already_commented = true;
                reset = false;
            }
            // bodhi has reset karma, so old comments can be disregarded
            if comment.user.name == "bodhi" && comment.text.contains("Karma") && comment.text.contains("reset") {
                already_commented = false;
                reset = true;
            }
        });

        (already_commented, reset)
    } else {
        (false, false)
    }
}

fn packages_in_update(update: &Update) -> Vec<String> {
    let names: Vec<String> = update
        .builds
        .iter()
        .map(|build| {
            build
                .nvr
                .parse::<NVR>()
                .expect("Failed to parse a build NVR from bodhi, this should not happen.")
                .n
        })
        .collect();
    names
}

#[tokio::main]
async fn main() -> Result<(), String> {
    // set up logger for warnings / debug messages
    // turn off very verbose rustyline debug logging
    #[cfg(not(feature = "debug"))]
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_env("FUF_LOG")
        .filter_module("rustyline", log::LevelFilter::Off)
        .init();
    #[cfg(feature = "debug")]
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .parse_env("FUF_LOG")
        .filter_module("rustyline", log::LevelFilter::Off)
        .init();

    let args: Command = Command::parse();

    let mut ignored = if !args.clear_ignored {
        match get_ignored().await {
            Ok(ignored) => ignored,
            Err(_) => IgnoreLists::default(),
        }
    } else {
        IgnoreLists::default()
    };

    if let Some(package) = &args.add_ignored_package {
        if !ignored.ignored_packages.contains(package) {
            println!("Added '{}' to the list of ignored packages.", &package);
            ignored.ignored_packages.push(package.clone());
            ignored.ignored_updates.sort();
            set_ignored(&ignored).await?;
        } else {
            println!("Already in the list of ignored packages: '{}'", &package);
        };
    }

    if let Some(package) = &args.remove_ignored_package {
        if ignored.ignored_packages.contains(package) {
            println!("Removed '{}' from the list of ignored packages.", &package);
            ignored.ignored_packages.retain(|p| p != package);
            set_ignored(&ignored).await?;
        } else {
            println!("Not in the list of ignored packages: '{}'", &package);
        };
    }

    if args.print_ignored {
        println!(
            "Ignored updates:{}",
            if ignored.ignored_updates.is_empty() {
                " none"
            } else {
                ""
            }
        );
        for update in &ignored.ignored_updates {
            println!("- {}", update);
        }
        println!();

        println!(
            "Ignored packages:{}",
            if ignored.ignored_packages.is_empty() {
                " none"
            } else {
                ""
            }
        );
        for package in &ignored.ignored_packages {
            println!("- {}", package);
        }
        println!();

        return Ok(());
    }

    if args.add_ignored_package.is_some() || args.remove_ignored_package.is_some() {
        return Ok(());
    }

    if !is_update_testing_enabled().await? {
        println!("WARNING: The 'updates-testing' repository does not seem to be enabled.");
        println!("         Usefulness of fedora-update-feedback will be limited.")
    }

    let config = get_config().await.ok();

    let username = if let Some(username) = &args.username {
        username.clone()
    } else if let Some(config) = &config {
        config.fas.username.clone()
    } else if let Ok(Some(username)) = get_legacy_username().await {
        username
    } else {
        return Err(String::from("Failed to read ~/.config/fedora.toml and ~/.fedora.upn."));
    };

    if args.verbose {
        println!("Username: {}", &username);
    }

    // read password from libsecret-1 or fall back to command line prompt
    let password = match &config {
        Some(config) => match &config.fuf {
            Some(fuf) => match fuf.save_password {
                Some(x) if x => get_store_password(args.ignore_keyring).await?,
                _ => read_password(),
            },
            None => read_password(),
        },
        None => read_password(),
    };

    if args.verbose {
        println!("Authenticating with bodhi ...");
    }
    let bodhi = BodhiClientBuilder::default()
        .user_agent(USER_AGENT)
        .authentication(&username, &password)
        .build()
        .await
        .map_err(|error| error.to_string())?;

    // query rpm for the current Fedora release number
    if args.verbose {
        println!("Querying RPM for the current Fedora release number ...");
    }
    let release = get_release().await?;

    // query DNF for installed packages
    if args.verbose {
        println!("Querying dnf for installed packages ...");
    }
    let installed_packages = get_installed().await?;

    // query DNF for source -> binary package map
    if args.verbose {
        println!("Querying dnf for mapping between source and binary packages ...");
    }
    let src_bin_map = get_src_bin_map().await?;

    // query dnf for package summaries
    if args.verbose {
        println!("Querying dnf for package summaries ...");
    }
    let summaries = get_summaries().await?;

    // query dnf for when the updates were installed
    if args.verbose {
        println!("Querying dnf for package installation times ...");
    }
    let install_times = get_installation_times().await?;

    // query bodhi for packages in updates-testing
    let mut updates: Vec<Update> = Vec::new();

    // get updates in "testing" state
    if args.verbose {
        println!("Querying bodhi for 'testing' updates ...");
    }
    let testing_updates = query_testing(&bodhi, release.clone()).await?;
    updates.extend(testing_updates);
    println!();

    if do_check_pending(&args, config.as_ref()) {
        // get updates in "pending" state
        if args.verbose {
            println!("Querying bodhi for 'pending' updates ...");
        }
        let pending_updates = query_pending(&bodhi, release.clone()).await?;
        updates.extend(pending_updates);
        println!();
    };

    if args.verbose {
        println!();
    }

    // filter out updates created by the current user
    let relevant_updates: Vec<Update> = updates
        .into_iter()
        .filter(|update| update.user.name != username)
        .collect();

    // filter out updates for packages that are not installed;
    // and remember which builds are installed for which update
    let mut installed_updates: Vec<&Update> = Vec::new();
    let mut builds_for_update: HashMap<String, Vec<String>> = HashMap::new();

    for update in &relevant_updates {
        let nvrs = update
            .builds
            .iter()
            .map(|b| b.nvr.parse())
            .collect::<Result<Vec<NVR>, String>>()?;

        for nvr in nvrs {
            if installed_packages.contains(&nvr) {
                installed_updates.push(update);

                builds_for_update
                    .entry(update.alias.clone())
                    .and_modify(|e| e.push(nvr.to_string()))
                    .or_insert_with(|| vec![nvr.to_string()]);
            };
        }
    }

    if installed_updates.is_empty() {
        println!("No updates that are waiting for feedback are currently installed.");

        if do_check_obsoletes(&args, config.as_ref()) {
            obsoleted_check(
                &bodhi,
                release.clone(),
                &installed_packages,
                &src_bin_map,
                &mut builds_for_update,
            )
            .await?;
        };

        if do_check_unpushed(&args, config.as_ref()) {
            unpushed_check(
                &bodhi,
                release.clone(),
                &installed_packages,
                &src_bin_map,
                &mut builds_for_update,
            )
            .await?;
        };

        return Ok(());
    };

    // deduplicate updates with multiple builds
    installed_updates.sort_by(|a, b| a.alias.cmp(&b.alias));
    installed_updates.dedup_by(|a, b| a.alias == b.alias);

    // sort updates by submission date
    installed_updates.sort_by(|a, b| a.date_submitted.cmp(&b.date_submitted));

    // remove old updates from ignored list
    ignored
        .ignored_updates
        .retain(|i| installed_updates.iter().map(|u| &u.alias).any(|x| x == i));

    // filter out updates that exclusively contain permanently ignored packages
    installed_updates.retain(|update| {
        let names = packages_in_update(update);
        !names.iter().all(|name| ignored.ignored_packages.contains(name))
    });

    // filter out previously ignored updates
    if !args.check_ignored {
        installed_updates.retain(|update| !ignored.ignored_updates.contains(&update.alias));
    }

    // keep track of the number of installed relevant updates
    let total_updates = installed_updates.len();

    // iterate over the list of installed relevant updates
    for (update_number, update) in installed_updates.into_iter().enumerate() {
        let (prev_commented, karma_reset) = has_already_commented(update, &username);
        let prev_ignored = ignored.ignored_updates.contains(&update.alias);

        // skip updates that were already commented on and where no karma reset has happened
        if !args.check_commented && prev_commented && !karma_reset {
            continue;
        }

        let progress = Progress::new(update_number, total_updates, prev_commented, karma_reset, prev_ignored);

        // this unwrap is safe since we definitely inserted a value for every update earlier
        #[allow(clippy::unwrap_used)]
        let builds = builds_for_update.get(update.alias.as_str()).unwrap();

        let mut binaries: Vec<&str> = Vec::new();
        for build in builds {
            if let Some(list) = src_bin_map.get(build) {
                binaries.extend(list.iter().map(|s| s.as_str()));
            };
        }

        let feedback = ask_feedback(update, progress, &binaries, &summaries, &install_times).await?;

        match feedback {
            Feedback::Abort => {
                println!("Aborting.");
                println!();
                break;
            },
            Feedback::Ignore => {
                println!("Ignoring.");
                println!();
                ignored.ignored_updates.push(update.alias.clone());
                ignored.ignored_updates.sort();
                continue;
            },
            Feedback::Block => {
                println!("Permanently ignoring all packages from this update.");
                println!();
                let names = packages_in_update(update);
                ignored.ignored_packages.extend(names);
                ignored.ignored_packages.sort();
                continue;
            },
            Feedback::Skip => {
                println!("Skipping.");
                println!();
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

                let mut builder = CommentCreator::new(&update.alias).karma(karma);

                if let Some(text) = &comment {
                    builder = builder.text(text);
                };

                let bug_feedbacks: Vec<BugFeedbackData> = bug_feedback
                    .into_iter()
                    .map(|(id, karma)| BugFeedbackData::new(id, karma))
                    .collect();
                builder = builder.bug_feedback(&bug_feedbacks);

                let testcase_feedbacks: Vec<TestCaseFeedbackData> = testcase_feedback
                    .into_iter()
                    .map(|(name, karma)| TestCaseFeedbackData::new(name, karma))
                    .collect();
                builder = builder.testcase_feedback(&testcase_feedbacks);

                let new_comment: Result<NewComment, QueryError> = bodhi.request(&builder).await;

                match new_comment {
                    Ok(value) => {
                        println!("Comment created.");
                        print_server_messages(&value.caveats);
                    },
                    Err(error) => {
                        println!("{}", error);
                    },
                };
            },
        };
    }

    // update list of ignored updates
    if let Err(error) = set_ignored(&ignored).await {
        println!("Failed to write ignored updates to disk.");
        println!("{}", error);
    };

    if do_check_obsoletes(&args, config.as_ref()) {
        obsoleted_check(
            &bodhi,
            release.clone(),
            &installed_packages,
            &src_bin_map,
            &mut builds_for_update,
        )
        .await?;
    };

    if do_check_unpushed(&args, config.as_ref()) {
        unpushed_check(
            &bodhi,
            release.clone(),
            &installed_packages,
            &src_bin_map,
            &mut builds_for_update,
        )
        .await?;
    };

    Ok(())
}
