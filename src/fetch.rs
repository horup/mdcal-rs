pub fn fetch(url: &str) -> Result<String, String> {
    reqwest::blocking::get(url)
        .and_then(|response| response.error_for_status())
        .and_then(|response| response.text())
        .map_err(|error| format!("failed to download iCal: {error}"))
}
