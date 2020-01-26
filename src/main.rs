use std::collections::HashMap;

use bodhi::error::QueryError;
use bodhi::*;

use structopt::StructOpt;

use fedora_update_feedback::*;

#[derive(Debug, StructOpt)]
struct Command {
    /// Override or provide FAS username
    #[structopt(long, short)]
    username: Option<String>,
    /// Include updates in "pending" state
    #[structopt(long, short = "p")]
    with_pending: bool,
    /// Clear ignored updates
    #[structopt(long, short = "i")]
    clear_ignored: bool,
}

fn main() -> Result<(), String> {
    let args: Command = Command::from_args();

    // check possible username sources in decending priority:
    // CLI argument, fedora.toml config file, .fedora.upn legacy fallback
    let username = match (args.username, get_config(), get_legacy_username()) {
        // prefer username specified on command line, if it was specified
        (Some(username), _, _) => username,

        // otherwise, prefer username from fedora.toml
        (None, Ok(config), _) => config.fas.username,

        // if that didn't work, use fallback value from .fedora.upn
        (None, Err(_), Ok(Some(username))) => {
            println!("Failed to read ~/.config/fedora.toml, using fallback (~/.fedora.upn).");
            username
        },

        // if reading config file failed and .fedora.upn is missing, error out
        (None, Err(error), Ok(None)) => {
            return Err(format!("{}, and fallback (~/.fedora.upn) not found.", error));
        },

        // if reading both the config file and .fedora.upn failed, error out
        (None, Err(err1), Err(err2)) => {
            return Err(format!("{} and failed to read ~/.fedora.upn ({}).", err1, err2));
        },
    };

    println!("Username: {}", &username);

    // read password from command line
    let password = rpassword::prompt_password_stdout("FAS Password: ").unwrap();

    // query rpm for current release
    let release = get_release()?;

    // query DNF for installed packages
    println!("Querying dnf for installed packages ...");
    let installed_packages = get_installed()?;

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

    println!("Querying bodhi for updates ...");
    let mut updates: Vec<Update> = Vec::new();

    // get updates in "testing" state
    let testing_updates = query_testing(&bodhi, release)?;
    updates.extend(testing_updates);
    println!();

    if args.with_pending {
        // get updates in "pending" state
        let pending_updates = query_pending(&bodhi, release)?;
        updates.extend(pending_updates);
        println!();
    };

    // get updates in "unpushed" state
    let unpushed_updates = query_unpushed(&bodhi, release)?;
    println!();

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
    let mut builds_for_update: HashMap<&str, Vec<String>> = HashMap::new();

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
                    .entry(&update.alias)
                    .and_modify(|e| e.push(nvr.to_string()))
                    .or_insert(vec![nvr.to_string()]);
            };
        }
    }

    if installed_updates.is_empty() {
        return Ok(());
    };

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
                    .entry(&update.alias)
                    .and_modify(|e| e.push(nvr.to_string()))
                    .or_insert(vec![nvr.to_string()]);
            };
        }
    }

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

    for update in installed_updates {
        if ignored.contains(&update.alias) {
            println!("Skipping ignored update: {}", &update.alias);
            continue;
        };

        // this unwrap is safe since we definitely inserted a value for every update
        let builds = builds_for_update.get(update.alias.as_str()).unwrap();

        let feedback = ask_feedback(&mut rl, update, &builds)?;

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

    if !installed_unpushed.is_empty() {
        println!("There are unpushed updates installed on this system.");
        println!("It is recommended to run 'dnf distro-sync' to clean this up.");

        for update in installed_unpushed {
            println!(" - {}:", update.title);
            // this unwrap is safe since we definitely inserted a value for every update
            for build in builds_for_update.get(update.alias.as_str()).unwrap() {
                println!("   - {}", build);
            }
        }
    };

    if let Err(error) = set_ignored(&ignored) {
        println!("Failed to write ignored updates to disk.");
        println!("{}", error);
    };

    Ok(())
}
