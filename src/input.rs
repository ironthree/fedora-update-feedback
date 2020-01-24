use std::io::{stdin, stdout, Write};

use bodhi::{Karma, Update};

use super::print_update;

/// This enum contains all feedback information for an update that's been parsed from CLI input.
pub enum Feedback<'a> {
    /// With this input, providing feedback for the current update is cancelled.
    Cancel,
    /// With this input, the current update is skipped.
    Skip,
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
pub fn ask_feedback<'a>(rl: &mut rustyline::Editor<()>, update: &'a Update) -> Result<Feedback<'a>, String> {
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

                if !comment_lines.is_empty() {
                    // if both the last line and the current line are empty, break
                    if comment_lines.last().unwrap().is_empty() && line.is_empty() {
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

    Ok(Feedback::Values {
        comment,
        karma,
        bug_feedback,
        testcase_feedback,
    })
}
