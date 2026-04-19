use serde_json::Value;

pub fn json_field<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.get(key)
}
