use std::process;

use chrono::Datelike;
use clap::{CommandFactory, Parser, Subcommand};

mod calendar;
mod fetch;
mod i18n;
mod markdown;

#[derive(Parser)]
#[command(
    name = "mdcal",
    version,
    about = "A CLI application which retrieves the calendar and returns it as markdown"
)]
struct Cli {
    /// iCal feed URL
    #[arg(value_name = "ICAL_URL")]
    ical_url: String,

    /// Language code (e.g., en, da)
    #[arg(short, long, default_value = "en", global = true)]
    lang: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show the current year calendar from the iCal feed
    Year,
    /// Show the current month and next two months from the iCal feed
    Month3,
}

/// Renders the current year's calendar as markdown.
fn year_markdown(ical_text: &str, strings: &i18n::Strings) -> Result<String, String> {
    let year = chrono::Utc::now().year();
    let events = calendar::events(ical_text, year..=year)?;
    let months: Vec<(i32, u32)> = (1..=12).map(|m| (year, m)).collect();
    Ok(markdown::calendar_markdown(&events, &months, strings))
}

/// Renders the current month and next two months as markdown.
fn month3_markdown(ical_text: &str, strings: &i18n::Strings) -> Result<String, String> {
    let now = chrono::Utc::now().date_naive();
    let start_month = now.month();
    let year = now.year();

    let months: Vec<(i32, u32)> = (0..3)
        .map(|offset| {
            let month = start_month + offset;
            if month <= 12 {
                (year, month)
            } else {
                (year + 1, month - 12)
            }
        })
        .collect();

    let years: std::ops::RangeInclusive<i32> = months[0].0..=months.last().unwrap().0;

    let events = calendar::events(ical_text, years)?;
    Ok(markdown::calendar_markdown(&events, &months, strings))
}

fn main() {
    let cli = Cli::parse();

    let strings = i18n::get(&cli.lang);

    let ical_text = fetch::fetch(&cli.ical_url).unwrap_or_else(|error| {
        eprintln!("error: {error}");
        process::exit(1);
    });

    match cli.command {
        Some(Commands::Year) => {
            let markdown = year_markdown(&ical_text, &strings).unwrap_or_else(|error| {
                eprintln!("error: {error}");
                process::exit(1);
            });

            print!("{markdown}");
        }
        Some(Commands::Month3) => {
            let markdown = month3_markdown(&ical_text, &strings).unwrap_or_else(|error| {
                eprintln!("error: {error}");
                process::exit(1);
            });

            print!("{markdown}");
        }
        None => {
            Cli::command().print_help().expect("failed to print help");
            println!();
        }
    }
}
