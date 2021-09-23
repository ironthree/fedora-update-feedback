use std::collections::HashMap;
use std::io::{stdin, stdout, Write};

use bodhi::{Karma, Update};
use chrono::{DateTime, Utc};

use crate::output::print_update;

/// This struct represents the user's progress through the list of installed updates.
#[derive(Debug)]
pub struct Progress {
    update_no: usize,
    no_updates: usize,
    no_ignored: usize,
}

impl Progress {
    pub fn new(update_no: usize, no_updates: usize, no_ignored: usize) -> Progress {
        Progress {
            update_no,
            no_updates,
            no_ignored,
        }
    }
}

/// This enum contains all feedback information for an update that's been parsed from CLI input.
pub enum Feedback<'a> {
    /// Cancel providing feedback for the current update.
    Cancel,
    /// Ignore this update now and in the future.
    Ignore,
    /// With this input, the current update is skipped.
    Skip,
    /// Cancel providing feedback for all remaining updates.
    Abort,
    /// These values are used when submitting feedback.
    Values {
        /// comment text (can be multiple lines)
        comment: Option<String>,
        /// feedback karma
        karma: Karma,
        /// list of bug feedback items (if any)
        bug_feedback: Vec<(u32, Karma)>,
        /// list of testcase feedback items (if any)
        testcase_feedback: Vec<(&'a str, Karma)>,
    },
}

/// This helper function prints a prompt and reads a string from standard input.
pub fn get_input(prompt: &str) -> String {
    let mut value = String::new();

    print!("{}: ", prompt);
    stdout().flush().expect("Failed to print prompt to stdout.");

    stdin().read_line(&mut value).expect("Failed to read from stdin.");

    value.trim().to_string()
}

/// This helper parses a string into an optional karma value instead of an error.
pub fn str_to_karma(string: &str) -> Option<Karma> {
    match string.parse() {
        Ok(karma) => Some(karma),
        Err(_) => None,
    }
}

/// This helper function prompts for all feedback values for a given update.
///
/// This includes:
///  - a prompt whether to skip the current update,
///  - text feedback (can be multiple lines, leading and trailing whitespace will be stripped
///    automatically; two empty lines or EOF (`Ctrl-D`) ends comment input)
///
/// If enabled at compile time, it also asks for bug and testcase feedback.
pub fn ask_feedback<'a>(
    rl: &mut rustyline::Editor<()>,
    update: &'a Update,
    progress: Progress,
    builds: &[&str],
    summaries: &HashMap<String, String>,
    install_times: &HashMap<String, DateTime<Utc>>,
) -> Result<Feedback<'a>, String> {
    print_update(update, builds, summaries, install_times);

    enum Action {
        Skip,
        Ignore,
        Abort,
        Comment,
    }

    println!(
        "This is update {} out of {} available updates (including {} ignored updates).",
        progress.update_no + 1,
        progress.no_updates,
        progress.no_ignored,
    );

    let action = match get_input("Action ([S]kip / [i]gnore / [c]omment / [a]bort)")
        .to_lowercase()
        .as_str()
    {
        "s" => Action::Skip,
        "i" => Action::Ignore,
        "c" => Action::Comment,
        "a" => Action::Abort,
        _ => Action::Skip,
    };

    if let Action::Skip = action {
        return Ok(Feedback::Skip);
    };

    if let Action::Ignore = action {
        return Ok(Feedback::Ignore);
    };

    if let Action::Abort = action {
        return Ok(Feedback::Abort);
    }

    println!("Add a descriptive comment (two empty lines or EOF (Ctrl-D) end input):");
    let mut comment_lines: Vec<String> = Vec::new();

    loop {
        match rl.readline("Comment: ") {
            Ok(line) => {
                rl.add_history_entry(&line);

                if !comment_lines.is_empty() {
                    // if both the last line and the current line are empty, break
                    if comment_lines
                        .last()
                        .expect("Something went wrong. There must be a last item in a non-empty iterable.")
                        .is_empty()
                        && line.is_empty()
                    {
                        break;
                    };
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

    let mut bug_feedback: Vec<(u32, Karma)> = Vec::new();
    for bug in &update.bugs {
        let bug_title = match &bug.title {
            Some(title) => title.as_str(),
            None => "(None)",
        };

        println!();
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
            println!();
            println!("{}", &test_case.name);

            if let Some(input) = str_to_karma(get_input("Test Case Feedback (+1, 0, -1)").as_str()) {
                testcase_feedback.push((&test_case.name, input));
            } else {
                println!("Skipped test case: {}", &test_case.name);
            };
        }
    }

    println!();

    Ok(Feedback::Values {
        comment,
        karma,
        bug_feedback,
        testcase_feedback,
    })
}
