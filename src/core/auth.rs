use actix_web::web;

pub fn validate_token(token: &web::Data<String>, query: &str) -> bool {
    token.get_ref() == query
}

pub fn extract_token_from_query(query: &str) -> Option<&str> {
    for pair in query.split('&') {
        if let Some(v) = pair.strip_prefix("access_token=") {
            return Some(v);
        }
    }
    None
}
