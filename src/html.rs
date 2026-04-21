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
    let day_num = date.day().to_string();
    let weekday_with_num = format!("{} {}", weekday, day_num);
    if date.weekday().number_from_monday() == 1 {
        let week_label = format!("[{} {}]", s.week, date.iso_week().week());
        (weekday_with_num, Some(week_label))
    } else {
        (weekday_with_num, None)
    }
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

    let mut widths = Vec::with_capacity(months.len());

    #[allow(unused_variables)]
    for (year, month) in &months {
        let width = month_name(strings, *month).len();
        widths.push(width);
    }

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n");
    html.push_str("<link href=\"https://fonts.googleapis.com/css2?family=Roboto+Mono&display=swap\" rel=\"stylesheet\">\n");
    html.push_str("<style>body { font-family: 'Roboto Mono', monospace; print-color-adjust: exact; -webkit-print-color-adjust: exact; } table { width: 100%; border-collapse: collapse; } td, th { border-bottom: 1px solid #ccc; padding: 4px; text-align: left; } th { background: #f5f5f5; } .weekend { background: #f0f0f0; }</style>\n");
    html.push_str("</head>\n<body>\n<table>\n<thead>\n<tr>\n");

    #[allow(unused_variables)]
    for (index, (year, month)) in months.iter().enumerate() {
        html.push_str("<th>");
        html.push_str(month_name(strings, *month));
        html.push_str("</th>\n");
    }
    html.push_str("</tr>\n</thead>\n<tbody>\n");

    for day in 1..=31 {
        html.push_str("<tr>\n");

        #[allow(unused_variables)]
        for (index, (year, month)) in months.iter().enumerate() {
            let cell_day = NaiveDate::from_ymd_opt(*year, *month, day);
            let is_weekend = cell_day
                .map(|d| d.weekday().number_from_monday() > 5)
                .unwrap_or(false);
            if is_weekend {
                html.push_str("<td class=\"weekend\">");
            } else {
                html.push_str("<td>");
            }
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
                    let padding =
                        widths[index].saturating_sub(weekday.len() + value.len() + label.len() + 2);
                    format!("{} {} {}{}", weekday, value, " ".repeat(padding), label)
                } else {
                    format!("{} {}", weekday, value)
                }
            } else if let Some(label) = week_label {
                let padding = widths[index].saturating_sub(weekday.len() + label.len() + 1);
                format!("{} {}{}", weekday, " ".repeat(padding), label)
            } else {
                weekday
            };

            html.push_str(&format!("{:<width$}", cell_content, width = widths[index]));
            html.push_str("</td>\n");
        }

        html.push_str("</tr>\n");
    }

    html.push_str("</tbody>\n</table>\n</body>\n</html>");

    html
}
