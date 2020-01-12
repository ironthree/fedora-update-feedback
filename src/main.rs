use std::cmp::PartialEq;
use std::convert::TryFrom;
use std::fs::read_to_string;
use std::io::{stdin, stdout, Write};
use std::process::Command;

use bodhi::*;

use bodhi::error::QueryError;
use serde::Deserialize;

#[derive(Debug, PartialEq)]
struct NVR {
    n: String,
    v: String,
    r: String,
}

#[derive(Debug, Deserialize)]
struct FedoraConfig {
    #[serde(rename(deserialize = "FAS"))]
    fas: FASConfig,
}

#[derive(Debug, Deserialize)]
struct FASConfig {
    username: String,
}

fn parse_nevra(nevra: &str) -> Result<(&str, &str, &str, &str, &str), String> {
    let mut nevr_a: Vec<&str> = nevra.rsplitn(2, '.').collect();

    if nevr_a.len() != 2 {
        return Err(format!("Unexpected error when parsing NEVRAs: {}", nevra));
    }

    // rsplitn returns things in reverse order
    let a = nevr_a.remove(0);
    let nevr = nevr_a.remove(0);

    let mut n_ev_r: Vec<&str> = nevr.rsplitn(3, '-').collect();

    if n_ev_r.len() != 3 {
        return Err(format!("Unexpected error when parsing NEVRAs: {}", nevr));
    }

    // rsplitn returns things in reverse order
    let r = n_ev_r.remove(0);
    let ev = n_ev_r.remove(0);
    let n = n_ev_r.remove(0);

    let (e, v) = if ev.contains(':') {
        let mut e_v: Vec<&str> = ev.split(':').collect();
        let e = e_v.remove(0);
        let v = e_v.remove(0);
        (e, v)
    } else {
        ("0", ev)
    };

    Ok((n, e, v, r, a))
}

fn parse_filename(nevrax: &str) -> Result<(&str, &str, &str, &str, &str), String> {
    let mut nevra_x: Vec<&str> = nevrax.rsplitn(2, '.').collect();

    if nevra_x.len() != 2 {
        return Err(format!("Unexpected error when parsing dnf output: {}", nevrax));
    }

    // rsplitn returns things in reverse order
    let _x = nevra_x.remove(0);
    let nevra = nevra_x.remove(0);

    let (n, e, v, r, a) = parse_nevra(nevra)?;
    Ok((n, e, v, r, a))
}

fn parse_nvr(nvr: &str) -> Result<(&str, &str, &str), String> {
    let mut n_v_r: Vec<&str> = nvr.rsplitn(3, '-').collect();

    if n_v_r.len() != 3 {
        return Err(format!("Unexpected error when parsing NEVRAs: {}", nvr));
    }

    // rsplitn returns things in reverse order
    let r = n_v_r.remove(0);
    let v = n_v_r.remove(0);
    let n = n_v_r.remove(0);

    Ok((n, v, r))
}

fn get_config() -> Result<FedoraConfig, String> {
    let home = match dirs::home_dir() {
        Some(path) => path,
        None => {
            return Err(String::from("Unable to determine $HOME."));
        },
    };

    let config_path = home.join(".config/fedora.toml");

    let config_str = match read_to_string(&config_path) {
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

fn get_release() -> Result<FedoraRelease, String> {
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

    let release = match FedoraRelease::try_from(release.as_str()) {
        Ok(release) => release,
        Err(error) => {
            return Err(error.to_string());
        },
    };

    Ok(release)
}

fn get_installed() -> Result<Vec<NVR>, String> {
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

enum Feedback<'a> {
    Cancel,
    Skip,
    Values {
        comment: Option<String>,
        karma: Karma,
        bug_feedback: Vec<(u32, Karma)>,
        testcase_feedback: Vec<(&'a str, Karma)>,
    },
}

fn get_input(prompt: &str) -> String {
    let mut value = String::new();

    print!("{}: ", prompt);
    stdout().flush().expect("Failed to print prompt to stdout.");

    stdin().read_line(&mut value).expect("Failed to read from stdin.");

    value.trim().to_string()
}

fn str_to_karma(string: &str) -> Option<Karma> {
    match string {
        "+1" => Some(Karma::Positive),
        "0" => Some(Karma::Neutral),
        "-1" => Some(Karma::Negative),
        _ => None,
    }
}

fn ask_feedback<'a>(rl: &mut rustyline::Editor<()>, update: &'a Update) -> Result<Feedback<'a>, String> {
    print_update(update);

    let skip = match get_input("Skip (Y/n)").as_str() {
        "y" | "Y" => true,
        "n" | "N" => false,
        _ => true,
    };

    if skip {
        return Ok(Feedback::Skip);
    };

    println!("Add a descriptive comment (two empty lines or EOF (Ctrl-D) end input):");
    let mut comment_lines: Vec<String> = Vec::new();

    loop {
        match rl.readline("Comment: ") {
            Ok(line) => {
                rl.add_history_entry(&line);

                if comment_lines.len() >= 1 {
                    // if both the last line and the current line are empty, break
                    if comment_lines.last().unwrap().is_empty() && line.is_empty() {
                        break;
                    }
                };

                comment_lines.push(line);
            },
            Err(rustyline::error::ReadlineError::Eof) => break,
            Err(rustyline::error::ReadlineError::Interrupted) => return Ok(Feedback::Cancel),
            Err(error) => return Err(error.to_string()),
        }
    }

    let comment = match comment_lines.join("\n").trim() {
        "" => None,
        x => Some(x.to_string()),
    };

    let karma = str_to_karma(get_input("Karma (+1, 0, -1)").as_str());

    if let (None, None) = (&comment, &karma) {
        println!("Provided neither comment nor karma, skipping this update.");
        return Ok(Feedback::Skip);
    };

    let karma = match karma {
        Some(karma) => karma,
        None => Karma::Neutral,
    };

    #[cfg(feature = "feedback")]
    {
        let mut bug_feedback: Vec<(u32, Karma)> = Vec::new();
        for bug in &update.bugs {
            let bug_title = match &bug.title {
                Some(title) => title.as_str(),
                None => "(None)",
            };

            println!("{}: {}", bug.bug_id, bug_title);
            if let Some(input) = str_to_karma(get_input("Bug Feedback (+1, 0, -1)").as_str()) {
                bug_feedback.push((bug.bug_id, input));
            } else {
                println!("Skipped bug: {}", bug.bug_id);
            };
        }

        let mut testcase_feedback: Vec<(&str, Karma)> = Vec::new();
        if let Some(test_cases) = &update.test_cases {
            for test_case in test_cases {
                println!("{}", &test_case.name);

                if let Some(input) = str_to_karma(get_input("Test Case Feedback (+1, 0, -1)").as_str()) {
                    testcase_feedback.push((&test_case.name, input));
                } else {
                    println!("Skipped test case: {}", &test_case.name);
                };
            }
        }

        return Ok(Feedback::Values {
            comment,
            karma,
            bug_feedback,
            testcase_feedback,
        });
    }

    #[cfg(not(feature = "feedback"))]
    Ok(Feedback::Values {
        comment,
        karma,
        bug_feedback: Vec::new(),
        testcase_feedback: Vec::new(),
    })
}

fn print_update(update: &Update) {
    let date = match &update.date_submitted {
        Some(date) => date.to_string(),
        None => "(None)".to_string(),
    };

    let karma = match update.karma {
        Some(karma) => karma.to_string(),
        None => "?".to_string(),
    };

    let stable_karma = match update.stable_karma {
        Some(karma) => karma.to_string(),
        None => "?".to_string(),
    };

    let unstable_karma = match update.unstable_karma {
        Some(karma) => karma.to_string(),
        None => "?".to_string(),
    };

    println!();

    println!("{}", "#".repeat(&update.alias.len() + 6));
    println!("## {} ##", update.alias);
    println!("{}", "#".repeat(&update.alias.len() + 6));

    println!();
    println!("{}", update.notes);
    println!();

    println!(
        "URL:            https://bodhi.fedoraproject.org/updates/{}",
        &update.alias
    );
    println!("Update type:    {}", update.update_type);
    println!("Submitted:      {}", date);
    println!("Submitter:      {}", update.user.name);
    println!("Karma:          {}", karma);
    println!("Stable karma:   {}", stable_karma);
    println!("Unstable karma: {}", unstable_karma);

    println!();

    if !update.bugs.is_empty() {
        let bugs: Vec<(u32, &str)> = update
            .bugs
            .iter()
            .map(|b| {
                (
                    b.bug_id,
                    match &b.title {
                        Some(title) => title.as_str(),
                        None => "(None)",
                    },
                )
            })
            .collect();

        println!("Bugs:");

        for (id, title) in bugs {
            println!("- {}: {}", id, title);
        }

        println!();
    };

    match &update.test_cases {
        Some(ts) if !ts.is_empty() => {
            let test_cases: Vec<&str> = ts.iter().map(|t| t.name.as_str()).collect();

            println!("Test cases:");

            for name in test_cases {
                println!("- {}", name);
            }

            println!();
        },
        _ => {},
    };

    println!("Builds:");
    for build in &update.builds {
        println!("- {}", &build.nvr);
    }

    println!();
}

fn main() -> Result<(), String> {
    let config = get_config()?;
    let username = config.fas.username;

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

    let progress_bar = |p: u32, ps: u32| {
        let width: u32 = 80 - 6;

        let progress = ((p as f64) / (ps as f64) * (width as f64)) as u32;
        let remaining = width - progress;

        let bar = format!(
            " [ {}{} ] ",
            "=".repeat(progress as usize),
            " ".repeat(remaining as usize)
        );

        print!("\r{}", &bar);
        std::io::stdout().flush().unwrap();
    };

    let testing_query = bodhi::query::UpdateQuery::new()
        .releases(release)
        .content_type(ContentType::RPM)
        .status(UpdateStatus::Testing)
        .callback(progress_bar);

    let testing_updates = match bodhi.query(testing_query) {
        Ok(updates) => updates,
        Err(error) => {
            return Err(format!("{}", error));
        },
    };

    println!();

    let pending_query = bodhi::query::UpdateQuery::new()
        .releases(release)
        .content_type(ContentType::RPM)
        .status(UpdateStatus::Pending)
        .callback(progress_bar);

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
                match (&comment, karma) {
                    (None, Karma::Neutral) => {
                        println!("Provided neither a comment nor karma feedback, skipping update.");
                        continue;
                    },
                    _ => {},
                }

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
