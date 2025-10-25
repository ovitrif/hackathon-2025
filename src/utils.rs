/// In this context, the title is the readable text on the 1st line
pub fn extract_title(input: &str) -> &str {
    // Get the first line by splitting on newlines and taking the first element
    let first_line = input.lines().next().unwrap_or("");
    first_line.trim_start_matches("# ")
}

pub fn extract_details_wiki_url(url: &str) -> Option<(String, String)> {
    // Split once on '/' and collect the two parts.
    let mut parts = url.splitn(2, '/');

    let first = parts.next()?.trim();
    let second = parts.next()?.trim();

    // Ensure both parts are present and not empty.
    if first.is_empty() || second.is_empty() {
        log::warn!("Invalid Pubky Wiki link: {url}");
        return None;
    }

    Some((first.to_string(), second.to_string()))
}
