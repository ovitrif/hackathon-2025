use regex::Regex;

/// Converts custom wiki link format (display_text)[userid/pageid] to standard markdown [display_text](userid/pageid)
pub fn convert_custom_links(content: &str) -> String {
    let re = Regex::new(r"\(([^)]+)\)\[([^\]]+)\]").unwrap();
    re.replace_all(content, "[$1]($2)").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_custom_links() {
        let input = "Check out (Alice's Page)[alice_user_id/550e8400-e29b-41d4-a716-446655440000]";
        let expected = "Check out [Alice's Page](alice_user_id/550e8400-e29b-41d4-a716-446655440000)";
        assert_eq!(convert_custom_links(input), expected);
    }

    #[test]
    fn test_multiple_links() {
        let input = "(Link 1)[page1] and (Link 2)[page2]";
        let expected = "[Link 1](page1) and [Link 2](page2)";
        assert_eq!(convert_custom_links(input), expected);
    }
}

