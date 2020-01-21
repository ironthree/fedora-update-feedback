use bodhi::error::QueryError;
use bodhi::*;

use fedora_update_feedback::*;

fn main() -> Result<(), String> {
    let username = match (get_config(), get_legacy_username()) {
        (Ok(config), _) => config.fas.username,
        (Err(_), Ok(Some(username))) => {
            println!("Failed to read ~/.config/fedora.toml, using fallback (~/.fedora.upn).");
            username
        },
        (Err(error), Ok(None)) => {
            return Err(format!("{}, and fallback (~/.fedora.upn) not found.", error));
        },
        (Err(err1), Err(err2)) => {
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
    let packages = get_installed()?;

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

    let testing = "Updates (testing)";
    let testing_progress = |p, ps| progress_bar(testing, p, ps);

    let testing_query = bodhi::query::UpdateQuery::new()
        .releases(release)
        .content_type(ContentType::RPM)
        .status(UpdateStatus::Testing)
        .callback(testing_progress);

    let testing_updates = match bodhi.query(testing_query) {
        Ok(updates) => updates,
        Err(error) => {
            return Err(format!("{}", error));
        },
    };

    println!();

    let pending = "Updates (pending)";
    let pending_progress = |p, ps| progress_bar(pending, p, ps);

    let pending_query = bodhi::query::UpdateQuery::new()
        .releases(release)
        .content_type(ContentType::RPM)
        .status(UpdateStatus::Pending)
        .callback(pending_progress);

    let pending_updates = match bodhi.query(pending_query) {
        Ok(updates) => updates,
        Err(error) => {
            return Err(format!("{}", error));
        },
    };

    println!();

    let mut updates: Vec<Update> = Vec::new();
    updates.extend(testing_updates);
    updates.extend(pending_updates);

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
                }
            }

            if !commented {
                relevant_updates.push(update);
            }
        } else {
            relevant_updates.push(update);
        }
    }

    // filter out updates for packages that are not installed
    let mut installed_updates: Vec<&Update> = Vec::new();
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
            if packages.contains(&nvr) {
                installed_updates.push(&update);
            }
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

    for update in installed_updates {
        let feedback = ask_feedback(&mut rl, update)?;

        match feedback {
            Feedback::Cancel => {
                println!("Cancelling.");
                break;
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
                }
            },
        };
    }

    Ok(())
}
