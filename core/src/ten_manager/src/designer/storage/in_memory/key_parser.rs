//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::{anyhow, Result};
use regex::Regex;
use serde_json::{Map, Value};

#[derive(Debug, Clone)]
pub enum KeySegment {
    Object(String),
    Array(String, usize),
}

pub fn parse_key(key: &str) -> Result<Vec<KeySegment>> {
    let mut segments = Vec::new();

    // Validate that key only contains allowed characters
    let valid_key_regex = Regex::new(r"^[a-z0-9_.\[\]]+$").unwrap();
    if !valid_key_regex.is_match(key) {
        return Err(anyhow!(
            "Key contains invalid characters. Only lowercase letters, \
             numbers, underscore, dots and brackets are allowed"
        ));
    }

    // Regex to match key segments: either "word" or "word[number]"
    let segment_regex = Regex::new(r"([a-z0-9_]+)(?:\[(\d+)\])?").unwrap();

    let parts: Vec<&str> = key.split('.').collect();

    for part in parts {
        if part.is_empty() {
            return Err(anyhow!("Empty key segment found"));
        }

        if let Some(captures) = segment_regex.captures(part) {
            let field_name = captures.get(1).unwrap().as_str().to_string();

            if let Some(index_match) = captures.get(2) {
                let index: usize = index_match
                    .as_str()
                    .parse()
                    .map_err(|_| anyhow!("Invalid array index"))?;
                segments.push(KeySegment::Array(field_name, index));
            } else {
                segments.push(KeySegment::Object(field_name));
            }
        } else {
            return Err(anyhow!("Invalid key segment: {}", part));
        }
    }

    if segments.is_empty() {
        return Err(anyhow!("No valid key segments found"));
    }

    Ok(segments)
}

pub fn get_value_by_key(data: &Value, key: &str) -> Result<Option<Value>> {
    let segments = parse_key(key)?;
    let mut current = data;

    for segment in &segments {
        match segment {
            KeySegment::Object(field) => {
                if let Some(obj) = current.as_object() {
                    current = match obj.get(field) {
                        Some(value) => value,
                        None => return Ok(None),
                    };
                } else {
                    return Ok(None);
                }
            }
            KeySegment::Array(field, index) => {
                if let Some(obj) = current.as_object() {
                    if let Some(arr_value) = obj.get(field) {
                        if let Some(arr) = arr_value.as_array() {
                            current = match arr.get(*index) {
                                Some(value) => value,
                                None => return Ok(None),
                            };
                        } else {
                            return Ok(None);
                        }
                    } else {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
            }
        }
    }

    Ok(Some(current.clone()))
}

pub fn set_value_by_key(
    data: &mut Value,
    key: &str,
    value: Value,
) -> Result<()> {
    let segments = parse_key(key)?;

    // Ensure root is an object
    if !data.is_object() {
        *data = Value::Object(Map::new());
    }

    set_value_recursive(data, &segments, 0, value)
}

fn set_value_recursive(
    current: &mut Value,
    segments: &[KeySegment],
    index: usize,
    value: Value,
) -> Result<()> {
    if index >= segments.len() {
        return Ok(());
    }

    let is_last = index == segments.len() - 1;
    let segment = &segments[index];

    match segment {
        KeySegment::Object(field) => {
            if is_last {
                if let Some(obj) = current.as_object_mut() {
                    obj.insert(field.clone(), value);
                    return Ok(());
                }
            } else if let Some(obj) = current.as_object_mut() {
                let entry = obj.entry(field.clone()).or_insert_with(|| {
                    // Look ahead to see if next segment is array
                    if let Some(next_segment) = segments.get(index + 1) {
                        match next_segment {
                            KeySegment::Array(_, _) => {
                                Value::Object(Map::new())
                            }
                            KeySegment::Object(_) => Value::Object(Map::new()),
                        }
                    } else {
                        Value::Object(Map::new())
                    }
                });
                return set_value_recursive(entry, segments, index + 1, value);
            }
        }
        KeySegment::Array(field, array_index) => {
            if let Some(obj) = current.as_object_mut() {
                let arr_entry = obj
                    .entry(field.clone())
                    .or_insert_with(|| Value::Array(Vec::new()));

                if let Some(arr) = arr_entry.as_array_mut() {
                    // Extend array if necessary
                    while arr.len() <= *array_index {
                        arr.push(Value::Null);
                    }

                    if is_last {
                        arr[*array_index] = value;
                        return Ok(());
                    } else {
                        // Ensure the array element is an object for further
                        // navigation
                        if !arr[*array_index].is_object() {
                            arr[*array_index] = Value::Object(Map::new());
                        }
                        return set_value_recursive(
                            &mut arr[*array_index],
                            segments,
                            index + 1,
                            value,
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_simple_key() {
        let result = parse_key("graph_ui").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            KeySegment::Object(name) => assert_eq!(name, "graph_ui"),
            _ => panic!("Expected object segment"),
        }
    }

    #[test]
    fn test_parse_nested_key() {
        let result = parse_key("graph_ui.test_graph").unwrap();
        assert_eq!(result.len(), 2);
        match &result[0] {
            KeySegment::Object(name) => assert_eq!(name, "graph_ui"),
            _ => panic!("Expected object segment"),
        }
        match &result[1] {
            KeySegment::Object(name) => assert_eq!(name, "test_graph"),
            _ => panic!("Expected object segment"),
        }
    }

    #[test]
    fn test_parse_array_key() {
        let result = parse_key("nodes[0].position").unwrap();
        assert_eq!(result.len(), 2);
        match &result[0] {
            KeySegment::Array(name, index) => {
                assert_eq!(name, "nodes");
                assert_eq!(*index, 0);
            }
            _ => panic!("Expected array segment"),
        }
        match &result[1] {
            KeySegment::Object(name) => assert_eq!(name, "position"),
            _ => panic!("Expected object segment"),
        }
    }

    #[test]
    fn test_invalid_key_uppercase() {
        assert!(parse_key("Graph_ui").is_err());
    }

    #[test]
    fn test_invalid_key_special_chars() {
        assert!(parse_key("graph-ui").is_err());
    }

    #[test]
    fn test_get_value_simple() {
        let data = json!({"graph_ui": {"test": "value"}});
        let result = get_value_by_key(&data, "graph_ui").unwrap();
        assert_eq!(result, Some(json!({"test": "value"})));
    }

    #[test]
    fn test_get_value_nested() {
        let data = json!({"graph_ui": {"test": "value"}});
        let result = get_value_by_key(&data, "graph_ui.test").unwrap();
        assert_eq!(result, Some(json!("value")));
    }

    #[test]
    fn test_get_value_array() {
        let data = json!({"nodes": [{"x": 10}, {"x": 20}]});
        let result = get_value_by_key(&data, "nodes[1].x").unwrap();
        assert_eq!(result, Some(json!(20)));
    }

    #[test]
    fn test_set_value_simple() {
        let mut data = json!({});
        set_value_by_key(&mut data, "test", json!("value")).unwrap();
        assert_eq!(data, json!({"test": "value"}));
    }

    #[test]
    fn test_set_value_nested() {
        let mut data = json!({});
        set_value_by_key(&mut data, "graph_ui.test", json!("value")).unwrap();
        assert_eq!(data, json!({"graph_ui": {"test": "value"}}));
    }

    #[test]
    fn test_set_value_array() {
        let mut data = json!({});
        set_value_by_key(&mut data, "nodes[1].x", json!(20)).unwrap();
        assert_eq!(data, json!({"nodes": [null, {"x": 20}]}));
    }
}
