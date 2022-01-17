use std::collections::HashMap;
use std::io::{stdout, Write};

use bodhi::{Comment, Update};
use chrono::{DateTime, Duration, Utc};

use crate::parse::parse_nvr;

/// This function draws a pretty progress bar with this format:
///
/// ` Prefix: [ =========                     ] 22% `
///
/// It's necessary to print a newline character after the progress bar reaches 100%.
pub fn progress_bar(prefix: &str, p: u32, ps: u32) {
    let columns: u32 = match term_size::dimensions() {
        Some((w, _)) => w as u32,
        None => return,
    };

    let width: u32 = columns - (prefix.len() as u32) - 13;

    let progress = ((p as f64) / (ps as f64) * (width as f64)) as u32;
    let remaining = width - progress;

    let line = format!(
        " {}: [ {}{} ] {:>3}% ",
        prefix,
        "=".repeat(progress as usize),
        " ".repeat(remaining as usize),
        ((p as f64) / (ps as f64) * 100f64) as u32,
    );

    print!("\r{}", &line);
    stdout().flush().expect("Failed to write to stdout.");
}

/// This helper function returns the duration from a datetime that lies in the past until now.
fn duration_until_now(datetime: &DateTime<Utc>) -> Duration {
    let result = Utc::now() - datetime.to_owned();

    if result <= Duration::seconds(0) {
        Duration::seconds(0)
    } else {
        result
    }
}

/// This helper handles proper pluralization of number terms.
pub(crate) fn proper_plural(number: i64, term: &str) -> String {
    if number == -1 || number == 1 {
        format!("{} {}", number, term)
    } else {
        format!("{} {}s", number, term)
    }
}

/// This helper function returns a pretty "duration" format.
fn pretty_duration(duration: Duration) -> String {
    if duration >= Duration::days(1) {
        let days = duration.num_days();
        let hours = (duration - Duration::days(days)).num_hours();
        format!("{} and {}", proper_plural(days, "day"), proper_plural(hours, "hour"))
    } else if duration >= Duration::hours(1) {
        let hours = duration.num_hours();
        let minutes = (duration - Duration::hours(hours)).num_minutes();
        format!(
            "{} and {}",
            proper_plural(hours, "hour"),
            proper_plural(minutes, "minute")
        )
    } else if duration >= Duration::minutes(1) {
        let minutes = duration.num_minutes();
        proper_plural(minutes, "minute")
    } else {
        String::from("less than a minute")
    }
}

/// This helper function pretty-prints an update.
pub fn print_update(
    update: &Update,
    builds: &[&str],
    summaries: &HashMap<String, String>,
    install_times: &HashMap<String, DateTime<Utc>>,
) {
    let submitted_date = match &update.date_submitted {
        Some(date) => date.to_string(),
        None => "(None)".to_string(),
    };

    let pushed_date = match &update.date_pushed {
        Some(date) => date.to_string(),
        None => "(not yet pushed)".to_string(),
    };

    let karma = match update.karma {
        Some(karma) => karma.to_string(),
        None => "?".to_string(),
    };

    let stable_karma = match update.stable_karma {
        Some(karma) => {
            if karma > 0 {
                format!("+{}", karma)
            } else {
                // this should never happen, but better print weird things than wrong things
                karma.to_string()
            }
        },
        None => "?".to_string(),
    };

    let unstable_karma = match update.unstable_karma {
        Some(karma) => karma.to_string(),
        None => "?".to_string(),
    };

    println!();

    // block for pretty-printing width-constrained strings
    match term_size::dimensions() {
        Some((w, _)) => {
            // construct a nice header banner for the update
            let boxie = "#".repeat(w);
            let header = if update.alias.len() > (w - 6) {
                update.alias.to_string()
            } else {
                let spaces = w - 4 - update.alias.len();
                let lspaces = " ".repeat(spaces / 2);
                let rspaces = " ".repeat((spaces / 2) + (spaces % 2));
                format!("##{}{}{}##", &lspaces, &update.alias, &rspaces)
            };
            let banner = format!("{}\n{}\n{}", &boxie, &header, &boxie);
            println!("{}", &banner);
            println!();

            // print human-readable update title
            println!("{}", textwrap::fill(&update.title, w - 1));

            let title_w = update.title.len();
            if title_w < w {
                println!("{}", "-".repeat(title_w));
            } else {
                println!("{}", "-".repeat(w));
            }
            println!();

            // print user-facing update notes
            println!("{}", textwrap::fill(update.notes.trim(), w - 1));
        },

        None => {
            println!("## {} ##", update.alias);
            println!();
            println!("{}", &update.title);
            println!();
            println!("{}", &update.notes);
        },
    }

    // block for rendering width-independent table
    println!();

    println!(
        "URL:            https://bodhi.fedoraproject.org/updates/{}",
        &update.alias
    );
    println!("Update type:    {}", update.update_type);
    println!("Submitted:      {}", submitted_date);
    println!("Pushed:         {}", pushed_date);
    println!("Submitter:      {}", update.user.name);
    println!("Karma:          {}", karma);
    println!("Stable karma:   {}", stable_karma);
    println!("Unstable karma: {}", unstable_karma);

    println!();

    if !update.bugs.is_empty() {
        let bugs: Vec<(String, Option<&String>)> = update
            .bugs
            .iter()
            .map(|b| (b.url().to_string(), b.title.as_ref()))
            .collect();

        println!("Associated bugs:");

        for (url, title) in bugs {
            println!("- {}", url);

            if let Some(title) = title {
                // make sure bug title doesn't contain words that are split across lines
                match term_size::dimensions() {
                    Some((w, _)) => {
                        println!("{}", textwrap::indent(&textwrap::fill(title.trim(), w - 3), "  "));
                    },
                    None => {
                        println!("  {}", title);
                    },
                };
            };
        }

        println!();
    };

    match &update.test_cases {
        Some(ts) if !ts.is_empty() => {
            let test_cases: Vec<String> = ts.iter().map(|t| t.url().to_string()).collect();

            println!("Associated test cases:");

            for url in test_cases {
                println!("- {}", url);
            }

            println!();
        },
        _ => {},
    };

    println!("Locally installed packages contained in this update:");
    for build in builds {
        let name = parse_nvr(build)
            .unwrap_or_else(|_| panic!("Failed to parse build NVR: {}", build))
            .0;
        let summary = summaries.get(name);
        let install_time = install_times.get(*build);

        println!("- {}", build);

        if let Some(string) = summary {
            println!("  {}", string);
        }

        if let Some(datetime) = install_time {
            println!("  installed {} ago", pretty_duration(duration_until_now(datetime)));
        }
    }

    if let Some(comments) = &update.comments {
        let mut sorted: Vec<&Comment> = comments.iter().filter(|c| c.user.name != "bodhi").collect();

        if !sorted.is_empty() {
            println!();
            println!("Previous comments:");

            sorted.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

            for comment in sorted {
                println!("- {} ({}): {}", &comment.user.name, &comment.timestamp, &comment.karma);

                let trimmed = comment.text.trim();
                match term_size::dimensions() {
                    Some((w, _)) => {
                        if !trimmed.is_empty() {
                            println!("{}", textwrap::indent(&textwrap::fill(trimmed, w - 3), "  "));
                        }
                    },
                    None => {
                        println!("{}", trimmed);
                    },
                };
            }
        }
    };

    println!();
}
