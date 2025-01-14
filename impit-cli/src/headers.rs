use std::collections::HashMap;

pub(crate) fn process_headers(headers: Vec<String>) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for header in headers {
        let mut parts = header.splitn(2, ':');
        let key = parts.next().unwrap().trim().to_string();
        let value = parts.next().unwrap().trim().to_string();
        map.insert(key, value);
    }

    map
}
