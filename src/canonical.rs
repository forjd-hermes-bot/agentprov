use anyhow::{Context, Result};
use serde_json::{Map, Value};

pub fn canonical_hash(value: &Value) -> Result<String> {
    let canonical = canonicalise(value);
    let bytes = serde_json::to_vec(&canonical).context("serialise canonical JSON")?;
    let hash = blake3::hash(&bytes);
    Ok(format!("blake3:{}", hash.to_hex()))
}

pub fn remove_field(value: &mut Value, field: &str) {
    if let Value::Object(map) = value {
        map.remove(field);
    }
}

pub fn canonicalise(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            for key in keys {
                if let Some(v) = map.get(key) {
                    sorted.insert(key.clone(), canonicalise(v));
                }
            }
            Value::Object(sorted)
        }
        Value::Array(values) => Value::Array(values.iter().map(canonicalise).collect()),
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonical_hash_is_independent_of_object_key_order() {
        let a = json!({"b": 2, "a": 1, "nested": {"z": true, "c": null}});
        let b = json!({"nested": {"c": null, "z": true}, "a": 1, "b": 2});
        assert_eq!(canonical_hash(&a).unwrap(), canonical_hash(&b).unwrap());
    }
}
