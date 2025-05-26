//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
pub mod get;
pub mod set;

use anyhow::Result;
use serde_json::Value;
use std::fs;

use crate::home::data::get_home_data_path;

/// Read the persistent storage data from disk
pub fn read_persistent_storage() -> Result<Value> {
    let path = get_home_data_path();

    if !path.exists() {
        return Ok(Value::Object(serde_json::Map::new()));
    }

    let content = fs::read_to_string(&path)?;
    let data: Value = serde_json::from_str(&content)?;

    Ok(data)
}

/// Write the persistent storage data to disk
pub fn write_persistent_storage(data: &Value) -> Result<()> {
    let path = get_home_data_path();

    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(data)?;
    fs::write(&path, content)?;
    Ok(())
}
