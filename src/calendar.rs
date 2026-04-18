use chrono::{Datelike, Duration, Months, NaiveDate};
use std::io::Cursor;

#[derive(Clone, Copy)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

pub struct RecurrenceRule {
    pub frequency: Frequency,
    pub interval: u32,
    pub count: Option<usize>,
    pub until: Option<NaiveDate>,
}

fn parse_ical_date(value: &str) -> Option<NaiveDate> {
    let date_value = value.split('T').next().unwrap_or(value);
    NaiveDate::parse_from_str(date_value, "%Y%m%d").ok()
}

pub fn parse_rrule(rrule_text: &str) -> Option<RecurrenceRule> {
    let mut frequency = None;
    let mut interval = 1;
    let mut count = None;
    let mut until = None;

    for part in rrule_text.split(';') {
        let (key, value) = part.split_once('=')?;
        match key {
            "FREQ" => {
                frequency = Some(match value {
                    "DAILY" => Frequency::Daily,
                    "WEEKLY" => Frequency::Weekly,
                    "MONTHLY" => Frequency::Monthly,
                    "YEARLY" => Frequency::Yearly,
                    _ => return None,
                });
            }
            "INTERVAL" => interval = value.parse().ok()?,
            "COUNT" => count = value.parse().ok(),
            "UNTIL" => until = parse_ical_date(value),
            _ => {}
        }
    }

    Some(RecurrenceRule {
        frequency: frequency?,
        interval,
        count,
        until,
    })
}

fn next_occurrence(date: NaiveDate, rule: &RecurrenceRule) -> Option<NaiveDate> {
    match rule.frequency {
        Frequency::Daily => date.checked_add_signed(Duration::days(i64::from(rule.interval))),
        Frequency::Weekly => date.checked_add_signed(Duration::weeks(i64::from(rule.interval))),
        Frequency::Monthly => date.checked_add_months(Months::new(rule.interval)),
        Frequency::Yearly => date.checked_add_months(Months::new(rule.interval.saturating_mul(12))),
    }
}

pub fn expand_rrule_events(start: NaiveDate, rrule_text: &str, year: i32) -> Vec<NaiveDate> {
    let Some(rule) = parse_rrule(rrule_text) else {
        return Vec::new();
    };

    let mut dates = Vec::new();
    let mut current = start;
    let mut seen = 0usize;

    loop {
        if let Some(until) = rule.until {
            if current > until {
                break;
            }
        }

        if current.year() == year {
            dates.push(current);
        }

        seen += 1;
        if let Some(count) = rule.count {
            if seen >= count {
                break;
            }
        }

        let Some(next) = next_occurrence(current, &rule) else {
            break;
        };

        current = next;
        if current.year() > year && rule.until.is_none() {
            break;
        }
    }

    dates
}

pub fn events(
    ical_text: &str,
    years: std::ops::RangeInclusive<i32>,
) -> Result<Vec<(NaiveDate, String)>, String> {
    let years: Vec<i32> = years.collect();
    let mut events = Vec::new();

    let parser = ical::IcalParser::new(Cursor::new(ical_text.as_bytes()));
    for calendar in parser {
        let calendar = calendar.map_err(|error| format!("failed to parse iCal: {error}"))?;

        for event in calendar.events {
            let mut summary = None;
            let mut date = None;
            let mut recurrence = None;

            for property in event.properties {
                match property.name.as_str() {
                    "SUMMARY" => summary = property.value,
                    "DTSTART" => {
                        if let Some(value) = property.value {
                            if let Some(parsed) = parse_ical_date(&value) {
                                if years.contains(&parsed.year()) {
                                    date = Some(parsed);
                                }
                            }
                        }
                    }
                    "RRULE" => recurrence = property.value,
                    _ => {}
                }
            }

            if let (Some(date), Some(summary)) = (date, &summary) {
                events.push((date, summary.clone()));
            }

            if let (Some(start), Some(rrule), Some(summary)) = (date, recurrence, summary) {
                for year in &years {
                    for occurrence in expand_rrule_events(start, &rrule, *year) {
                        events.push((occurrence, summary.clone()));
                    }
                }
            }
        }
    }

    events.sort_by_key(|(date, _)| *date);
    Ok(events)
}
