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
    stdout().flush().expect("Failed to write to stdout.");
}

/// This helper function pretty-prints an update.
pub fn print_update(update: &Update, builds: &[&str]) {
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
    match term_size::dimensions() {
        Some((w, _)) => println!("{}", textwrap::fill(&update.notes, w)),
        None => println!("{}", &update.notes),
    };
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
        let bugs: Vec<(String, Option<&String>)> = update
            .bugs
            .iter()
            .map(|b| (b.url().to_string(), b.title.as_ref()))
            .collect();

        println!("Bugs:");

        for (url, title) in bugs {
            println!("- {}", url);

            if let Some(title) = title {
                println!("  {}", title);
            };
        }

        println!();
    };

    match &update.test_cases {
        Some(ts) if !ts.is_empty() => {
            let test_cases: Vec<String> = ts.iter().map(|t| t.url().to_string()).collect();

            println!("Test cases:");

            for url in test_cases {
                println!("- {}", url);
            }

            println!();
        },
        _ => {},
    };

    println!("Builds:");
    for build in builds {
        println!("- {}", build);
    }

    println!();
}
