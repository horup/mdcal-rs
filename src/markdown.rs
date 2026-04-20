use chrono::{Datelike, NaiveDate};

use crate::i18n::Strings;

fn month_name(s: &Strings, month: u32) -> &'static str {
    s.months.get((month - 1) as usize).copied().unwrap_or("")
}

fn escape_html_cell(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\n', "<br>")
}

fn weekday_name(s: &Strings, date: NaiveDate) -> &'static str {
    let idx = date.weekday().number_from_monday() as usize - 1;
    s.weekdays.get(idx).copied().unwrap_or("")
}

fn day_prefix(s: &Strings, date: NaiveDate) -> (String, Option<String>) {
    let weekday = weekday_name(s, date).to_string();
    if date.weekday().number_from_monday() == 1 {
        let week_label = format!("[{}{}]", s.week, date.iso_week().week());
        (weekday, Some(week_label))
    } else {
        (weekday, None)
    }
}

fn pad_header(value: impl std::fmt::Display, width: usize) -> String {
    let value = value.to_string();
    if width <= value.len() {
        return value;
    }

    let padding = width - value.len();
    let left = padding / 2;
    let right = padding - left;
    format!("{}{}{}", " ".repeat(left), value, " ".repeat(right))
}

fn pad_cell(value: &str, width: usize) -> String {
    let value = value.to_string();
    if width <= value.len() {
        return value;
    }

    let padding = width - value.len();
    let left = padding / 2;
    let right = padding - left;
    format!("{}{}{}", " ".repeat(left), value, " ".repeat(right))
}

fn escape_markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', "<br>")
}

/// Renders calendar events as a markdown table.
///
/// `events` is a list of (date, summary) tuples.
/// `months` is a list of (year, month) pairs to display.
/// `strings` provides localized month and weekday names.
pub fn calendar_markdown(
    events: &[(NaiveDate, String)],
    months: &[(i32, u32)],
    strings: &Strings,
) -> String {
    use std::collections::BTreeMap;

    let months: Vec<(i32, u32)> = months.to_vec();
    let mut cells: BTreeMap<(u32, u32, i32), Vec<String>> = BTreeMap::new();

    for (date, summary) in events {
        if let Some(m) = months
            .iter()
            .find(|(y, m)| *y == date.year() && *m == date.month())
        {
            cells
                .entry((date.day(), m.1, m.0))
                .or_default()
                .push(summary.clone());
        }
    }

    let mut widths = Vec::with_capacity(months.len() + 1);
    widths.push(2usize.max(31usize.to_string().len()));

    #[allow(unused_variables)]
    for (year, month) in &months {
        let mut width = month_name(strings, *month).len();
        for day in 1..=31 {
            if let Some(cell_day) = NaiveDate::from_ymd_opt(*year, *month, day) {
                let (weekday, week_label) = day_prefix(strings, cell_day);
                let prefix_len = weekday.len() + week_label.as_ref().map(|w| w.len()).unwrap_or(0);
                if let Some(entries) = cells.get(&(day, *month, *year)) {
                    let value = entries
                        .iter()
                        .map(|entry| escape_markdown_cell(entry))
                        .collect::<Vec<_>>()
                        .join("<br>");
                    width = width.max(prefix_len + value.len() + 1);
                } else {
                    width = width.max(prefix_len);
                }
            }
        }
        widths.push(width);
    }

    let mut markdown = String::new();
    markdown.push('|');
    markdown.push(' ');
    markdown.push_str(&pad_header("", widths[0]));
    markdown.push(' ');
    markdown.push('|');

    #[allow(unused_variables)]
    for (index, (year, month)) in months.iter().enumerate() {
        markdown.push(' ');
        markdown.push_str(&pad_header(month_name(strings, *month), widths[index + 1]));
        markdown.push(' ');
        markdown.push('|');
    }
    markdown.push('\n');

    markdown.push('|');
    markdown.push(' ');
    markdown.push_str(&"-".repeat(widths[0]));
    markdown.push(' ');
    markdown.push('|');

    for width in widths.iter().skip(1) {
        markdown.push(' ');
        markdown.push_str(&"-".repeat(*width));
        markdown.push(' ');
        markdown.push('|');
    }
    markdown.push('\n');

    for day in 1..=31 {
        markdown.push('|');
        markdown.push(' ');
        markdown.push_str(&format!("{:<width$}", day, width = widths[0]));
        markdown.push(' ');
        markdown.push('|');

        #[allow(unused_variables)]
        for (index, (year, month)) in months.iter().enumerate() {
            markdown.push(' ');
            let cell_day = NaiveDate::from_ymd_opt(*year, *month, day);
            let (weekday, week_label) = cell_day
                .map(|d| day_prefix(strings, d))
                .unwrap_or((String::new(), None));

            let cell_content = if let Some(entries) = cells.get(&(day, *month, *year)) {
                let value = entries
                    .iter()
                    .map(|entry| escape_markdown_cell(entry))
                    .collect::<Vec<_>>()
                    .join(", ");
                if let Some(label) = week_label {
                    let padding = widths[index + 1]
                        .saturating_sub(weekday.len() + value.len() + label.len() + 1);
                    format!("{} {}{}{}", weekday, value, " ".repeat(padding), label)
                } else {
                    format!("{} {}", weekday, value)
                }
            } else {
                if let Some(label) = week_label {
                    let padding = widths[index + 1].saturating_sub(weekday.len() + label.len());
                    format!("{}{}{}", weekday, " ".repeat(padding), label)
                } else {
                    weekday
                }
            };

            markdown.push_str(&format!(
                "{:<width$}",
                cell_content,
                width = widths[index + 1]
            ));
            markdown.push(' ');
            markdown.push('|');
        }

        markdown.push('\n');
    }

    markdown
}

pub fn calendar_html(
    events: &[(NaiveDate, String)],
    months: &[(i32, u32)],
    strings: &Strings,
) -> String {
    use std::collections::BTreeMap;

    let months: Vec<(i32, u32)> = months.to_vec();
    let mut cells: BTreeMap<(u32, u32, i32), Vec<String>> = BTreeMap::new();

    for (date, summary) in events {
        if let Some(m) = months
            .iter()
            .find(|(y, m)| *y == date.year() && *m == date.month())
        {
            cells
                .entry((date.day(), m.1, m.0))
                .or_default()
                .push(summary.clone());
        }
    }

    let mut widths = Vec::with_capacity(months.len() + 1);
    widths.push(2usize.max(31usize.to_string().len()));

    #[allow(unused_variables)]
    for (year, month) in &months {
        let mut width = month_name(strings, *month).len();
        for day in 1..=31 {
            if let Some(cell_day) = NaiveDate::from_ymd_opt(*year, *month, day) {
                let (weekday, week_label) = day_prefix(strings, cell_day);
                let prefix_len = weekday.len() + week_label.as_ref().map(|w| w.len()).unwrap_or(0);
                if let Some(entries) = cells.get(&(day, *month, *year)) {
                    let value = entries
                        .iter()
                        .map(|entry| escape_html_cell(entry))
                        .collect::<Vec<_>>()
                        .join("<br>");
                    width = width.max(prefix_len + value.len() + 1);
                } else {
                    width = width.max(prefix_len);
                }
            }
        }
        widths.push(width);
    }

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n");
    html.push_str("<link href=\"https://fonts.googleapis.com/css2?family=Roboto&display=swap\" rel=\"stylesheet\">\n");
    html.push_str("<style>body { font-family: 'Roboto', sans-serif; } table { width: 100%; border-collapse: collapse; } td, th { border-bottom: 1px solid #ccc; padding: 4px; text-align: left; } th { background: #f5f5f5; }</style>\n");
    html.push_str("</head>\n<body>\n<table>\n<thead>\n<tr>\n<th></th>\n");

    #[allow(unused_variables)]
    for (index, (year, month)) in months.iter().enumerate() {
        html.push_str("<th>");
        html.push_str(&pad_cell(month_name(strings, *month), widths[index + 1]));
        html.push_str("</th>\n");
    }
    html.push_str("</tr>\n</thead>\n<tbody>\n");

    for day in 1..=31 {
        html.push_str("<tr>\n<td>");
        html.push_str(&format!("{:<width$}", day, width = widths[0]));
        html.push_str("</td>\n");

        #[allow(unused_variables)]
        for (index, (year, month)) in months.iter().enumerate() {
            html.push_str("<td>");
            let cell_day = NaiveDate::from_ymd_opt(*year, *month, day);
            let (weekday, week_label) = cell_day
                .map(|d| day_prefix(strings, d))
                .unwrap_or((String::new(), None));

            let cell_content = if let Some(entries) = cells.get(&(day, *month, *year)) {
                let value = entries
                    .iter()
                    .map(|entry| escape_html_cell(entry))
                    .collect::<Vec<_>>()
                    .join(", ");
                if let Some(label) = week_label {
                    let padding = widths[index + 1]
                        .saturating_sub(weekday.len() + value.len() + label.len() + 1);
                    format!("{} {}{}{}", weekday, value, " ".repeat(padding), label)
                } else {
                    format!("{} {}", weekday, value)
                }
            } else {
                if let Some(label) = week_label {
                    let padding = widths[index + 1].saturating_sub(weekday.len() + label.len());
                    format!("{}{}{}", weekday, " ".repeat(padding), label)
                } else {
                    weekday
                }
            };

            html.push_str(&format!(
                "{:<width$}",
                cell_content,
                width = widths[index + 1]
            ));
            html.push_str("</td>\n");
        }

        html.push_str("</tr>\n");
    }

    html.push_str("</tbody>\n</table>\n</body>\n</html>");

    html
}
