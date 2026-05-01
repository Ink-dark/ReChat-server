use rechat_sender::core::auth;

#[test]
fn test_validate_token_match() {
    assert!(auth::validate_token("secret123", "secret123"));
}

#[test]
fn test_validate_token_mismatch() {
    assert!(!auth::validate_token("secret123", "wrong"));
}

#[test]
fn test_validate_token_empty() {
    assert!(!auth::validate_token("secret123", ""));
}

#[test]
fn test_extract_token_from_query_found() {
    assert_eq!(
        auth::extract_token_from_query("access_token=abc123&other=1"),
        Some("abc123")
    );
}

#[test]
fn test_extract_token_from_query_only_token() {
    assert_eq!(
        auth::extract_token_from_query("access_token=xyz"),
        Some("xyz")
    );
}

#[test]
fn test_extract_token_from_query_not_found() {
    assert_eq!(auth::extract_token_from_query("other=1&foo=bar"), None);
}

#[test]
fn test_extract_token_from_query_empty() {
    assert_eq!(auth::extract_token_from_query(""), None);
}
