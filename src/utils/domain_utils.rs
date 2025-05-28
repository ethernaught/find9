pub fn split_domain(name: &str) -> Option<(String, String)> {
    let mut parts: Vec<&str> = name.trim_end_matches('.').split('.').collect();
    if parts.len() < 2 {
        return None;
    }

    let tld = parts.split_off(parts.len() - 2).join(".");
    let name = if parts.is_empty() {
        "@".to_string()
    } else {
        parts.join(".")
    };

    Some((name, tld))
}
