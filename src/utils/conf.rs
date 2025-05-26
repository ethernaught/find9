use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
enum ConfEntry {
    Block(HashMap<String, String>),
    Value(String),
}

type Config = HashMap<String, ConfEntry>;

fn parse_bind_config(path: &str) -> std::io::Result<Config> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines().peekable();

    let mut config = HashMap::new();

    while let Some(Ok(line)) = lines.next() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }

        if line.starts_with("zone") {
            // zone "example.com" {
            let name_start = line.find('"').unwrap() + 1;
            let name_end = line[name_start..].find('"').unwrap() + name_start;
            let zone_name = &line[name_start..name_end];

            // Skip to opening brace
            while !line.contains('{') {
                if let Some(Ok(next)) = lines.next() {
                    if next.contains('{') {
                        break;
                    }
                }
            }

            // Parse block
            let mut block = HashMap::new();
            while let Some(Ok(body_line)) = lines.next() {
                let body_line = body_line.trim();
                if body_line == "};" || body_line == "}" {
                    break;
                }

                if let Some((key, value)) = body_line.strip_suffix(';')
                    .and_then(|l| l.split_once(char::is_whitespace))
                {
                    let val = value.trim_matches('"');
                    block.insert(key.to_string(), val.to_string());
                }
            }

            config.insert(zone_name.to_string(), ConfEntry::Block(block));
        } else if let Some((key, value)) = line.strip_suffix(';')
            .and_then(|l| l.split_once(char::is_whitespace))
        {
            config.insert(key.to_string(), ConfEntry::Value(value.trim_matches('"').to_string()));
        }
    }

    Ok(config)
}
