fn main() {
    let backends: Vec<_> = std::env::vars()
        .filter_map(|(key, _value)| {
            if key.starts_with("CARGO_FEATURE_") && key.ends_with("_BACKEND") {
                Some(key[14..].to_ascii_lowercase()) // Strip 'CARGO_FEATURE_'
            } else {
                None
            }
        })
        .collect();

    if backends.is_empty() {
        panic!("No backend feature selected");
    } else if backends.len() > 1 {
        panic!("Multiple backend features selected: {:?}", backends);
    }
}
