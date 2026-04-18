pub struct Strings {
    pub months: [&'static str; 12],
    pub weekdays: [&'static str; 7],
    pub week: &'static str,
}

impl Default for Strings {
    fn default() -> Self {
        Self::en()
    }
}

impl Strings {
    pub fn en() -> Self {
        Self {
            months: [
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ],
            weekdays: ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"],
            week: "Week",
        }
    }
}

pub fn get(locale: &str) -> Strings {
    match locale {
        "da" => Strings {
            months: [
                "Januar",
                "Februar",
                "Marts",
                "April",
                "Maj",
                "Juni",
                "Juli",
                "August",
                "September",
                "Oktober",
                "November",
                "December",
            ],
            weekdays: ["Ma", "Ti", "On", "To", "Fr", "Lø", "Sø"],
            week: "Uge",
        },
        _ => Strings::en(),
    }
}
