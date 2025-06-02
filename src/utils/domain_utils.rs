pub fn split_domain(name: &str) -> Option<(String, String)> {
    let mut parts: Vec<&str> = name.trim_end_matches('.').split('.').collect();
    if parts.len() < 2 {
        return Some((String::from("@"), name.to_string()));
    }

    let tld = parts.split_off(parts.len() - 2).join(".");
    let name = if parts.is_empty() {
        String::from("@")
    } else {
        parts.join(".")
    };

    Some((name, tld))
}
