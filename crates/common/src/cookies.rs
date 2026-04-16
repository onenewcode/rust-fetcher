use std::collections::BTreeMap;

pub fn parse_cookie_string(input: &str) -> BTreeMap<String, String> {
    let mut parsed = BTreeMap::new();

    for chunk in input.split(';') {
        let trimmed = chunk.trim();
        if trimmed.is_empty() {
            continue;
        }

        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        parsed.insert(key.trim().to_string(), value.trim().to_string());
    }

    parsed
}

#[cfg(test)]
mod tests {
    use super::parse_cookie_string;

    #[test]
    fn ignores_blank_or_invalid_segments() {
        let parsed = parse_cookie_string("a=1; ; broken; b = 2");

        assert_eq!(parsed.get("a").map(String::as_str), Some("1"));
        assert_eq!(parsed.get("b").map(String::as_str), Some("2"));
        assert_eq!(parsed.len(), 2);
    }
}
