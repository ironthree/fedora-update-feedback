use std::io::{stdout, Write};

use bodhi::Update;

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
    stdout().flush().unwrap();
}

/// This helper function pretty-prints an update.
pub fn print_update(update: &Update) {
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
