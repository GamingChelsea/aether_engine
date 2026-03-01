use std::collections::HashMap;
use std::sync::RwLock;
use toml::Value;

static TRANSLATIONS: RwLock<Option<HashMap<String, String>>> = RwLock::new(None);

pub fn get(key: &str) -> String {
    let lock = TRANSLATIONS.read().unwrap();
    match &*lock {
        Some(map) => map.get(key).cloned().unwrap_or_else(|| key.to_string()),
        None => key.to_string(),
    }
}

pub fn load(path: &str) {
    let content = std::fs::read_to_string(path).expect("Locale File could not be read");

    let parsed: Value = toml::from_str(&content).expect("Locale File is not a valid TOML");

    let mut translations = HashMap::new();
    if let Some(table) = parsed.as_table() {
        flatten_table("", table, &mut translations);
    }

    let mut lock = TRANSLATIONS.write().unwrap();
    *lock = Some(translations);
}

fn flatten_table(
    prefix: &str,
    table: &toml::map::Map<String, Value>,
    map: &mut HashMap<String, String>,
) {
    for (key, value) in table {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };

        match value {
            Value::String(s) => {
                map.insert(full_key, s.clone());
            }
            Value::Table(t) => {
                flatten_table(&full_key, t, map);
            }
            _ => {
                map.insert(full_key, value.to_string());
            }
        }
    }
}

#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::get($key)
    };
}
