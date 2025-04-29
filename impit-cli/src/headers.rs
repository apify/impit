pub(crate) fn process_headers(headers: Vec<String>) -> Vec<(String, String)> {
    headers
        .iter()
        .map(|header| {
            let parts: Vec<&str> = header.split(':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                (key, value)
            } else {
                ("".to_string(), "".to_string())
            }
        })
        .collect()
}
