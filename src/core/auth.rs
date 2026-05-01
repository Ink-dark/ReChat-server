pub fn validate_token(expected: &str, provided: &str) -> bool {
    expected == provided
}

pub fn extract_token_from_query(query: &str) -> Option<&str> {
    for pair in query.split('&') {
        if let Some(v) = pair.strip_prefix("access_token=") {
            return Some(v);
        }
    }
    None
}
