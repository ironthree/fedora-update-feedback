use std::collections::HashMap;
use std::env;
use std::io::{stdin, stdout, Write};

use bodhi::{Karma, Update};
use chrono::{DateTime, Utc};
use tokio::process::Command;

use crate::output::print_update;

const DEFAULT_EDITOR: &str = "nano";

fn detect_editor() -> String {
    if let Ok(editor) = env::var("EDITOR") {
        editor
    } else if let Ok(editor) = env::var("VISUAL") {
        editor
    } else {
        DEFAULT_EDITOR.to_string()
    }
}

async fn get_comment_from_editor() -> Result<Option<String>, String> {
    let editor = detect_editor();

    let temp_file = tempfile::Builder::new()
        .suffix(".md")
        .tempfile()
        .map_err(|err| err.to_string())?;

    let mut cmd = Command::new(editor);
    cmd.arg(temp_file.path());
    cmd.status().await.map_err(|err| err.to_string())?;

    let output = tokio::fs::read_to_string(temp_file)
        .await
        .map_err(|err| err.to_string())?;

    match output.trim() {
        "" => Ok(None),
        _ => Ok(Some(output)),
    }
}

/// This struct represents the user's progress through the list of installed updates.
#[derive(Debug)]
pub struct Progress {
    update_number: usize,
    total_updates: usize,
    prev_commented: bool,
    karma_reset: bool,
    prev_ignored: bool,
}

impl Progress {
    pub fn new(
        update_number: usize,
        total_updates: usize,
        prev_commented: bool,
        karma_reset: bool,
        prev_ignored: bool,
    ) -> Progress {
        Progress {
            update_number,
            total_updates,
            prev_commented,
            karma_reset,
            prev_ignored,
        }
    }
}

/// This enum contains all feedback information for an update that's been parsed from CLI input.
pub enum Feedback<'a> {
    /// Ignore this update now and in the future.
    Ignore,
    /// With this input, the current update is skipped.
    Skip,
    /// Cancel providing feedback for all remaining updates.
    Abort,
    /// Add this package to the list of permanently ignored packages
    Block,
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
pub async fn ask_feedback<'a>(
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
        Block,
    }

    println!(
        "Updates considered: {}, Updates remaining: {}",
        progress.update_number,
        progress.total_updates - progress.update_number - 1
    );

    if progress.prev_commented {
        if progress.karma_reset {
            println!("A comment for this update has already been submitted.");
            println!("However, the update has since been edited, and karma has been reset.");
        } else {
            println!("A comment for this update has already been submitted.");
            println!("Any feedback / karma that is provided now will overwrite previous values.");
        }
    }

    if progress.prev_ignored {
        println!("This update has been previously marked as ignored.");
    }

    println!("Actions: [s] skip this update (default)");
    println!("       / [i] ignore this update permanently");
    println!("       / [c] comment with feedback (opens an external editor)");
    println!("       / [b] block (ignore all packages from this update permanently)");
    println!("       / [a] abort (exit program)");

    let action = match get_input("Action").to_lowercase().as_str() {
        "s" => Action::Skip,
        "i" => Action::Ignore,
        "c" => Action::Comment,
        "a" => Action::Abort,
        "b" => Action::Block,
        _ => Action::Skip,
    };

    if let Action::Skip = action {
        return Ok(Feedback::Skip);
    };

    if let Action::Ignore = action {
        return Ok(Feedback::Ignore);
    };

    if let Action::Block = action {
        return Ok(Feedback::Block);
    }

    if let Action::Abort = action {
        return Ok(Feedback::Abort);
    }

    let comment = get_comment_from_editor().await?;
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

    if comment.is_none() && karma == Karma::Neutral && bug_feedback.is_empty() && testcase_feedback.is_empty() {
        Ok(Feedback::Skip)
    } else {
        Ok(Feedback::Values {
            comment,
            karma,
            bug_feedback,
            testcase_feedback,
        })
    }
}
