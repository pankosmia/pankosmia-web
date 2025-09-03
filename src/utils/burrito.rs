use crate::structs::MetadataSummary;
use serde_json::{Map, Value};
use std::io;

pub(crate) fn summary_metadata_from_file(
    repo_metadata_path: String,
) -> Result<MetadataSummary, io::Error> {
    let file_string = match std::fs::read_to_string(&repo_metadata_path) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let raw_metadata_struct: Value = match serde_json::from_str(file_string.as_str()) {
        Ok(v) => v,
        Err(e) => {
            return Err(io::Error::from(e));
        }
    };
    let current_scope_values =
        match raw_metadata_struct["type"]["flavorType"]["currentScope"].as_object() {
            Some(v) => v,
            None => &Map::new(),
        };
    let mut book_codes = Vec::new();
    for (map_key, _) in current_scope_values.clone().iter() {
        book_codes.push(format!("{}", map_key));
    }
    Ok(MetadataSummary {
        name: raw_metadata_struct["identification"]["name"]["en"]
            .as_str()
            .unwrap()
            .to_string(),
        description: match raw_metadata_struct["identification"]["description"]["en"].clone() {
            Value::String(v) => v.as_str().to_string(),
            Value::Null => "".to_string(),
            _ => "?".to_string(),
        },
        abbreviation: match raw_metadata_struct["identification"]["abbreviation"]["en"].clone() {
            Value::String(v) => v.as_str().to_string(),
            Value::Null => "".to_string(),
            _ => "?".to_string(),
        },
        generated_date: match raw_metadata_struct["meta"]["dateCreated"].clone() {
            Value::String(v) => v.as_str().to_string(),
            Value::Null => "".to_string(),
            _ => "?".to_string(),
        },
        flavor_type: raw_metadata_struct["type"]["flavorType"]["name"]
            .as_str()
            .unwrap()
            .to_string(),
        flavor: raw_metadata_struct["type"]["flavorType"]["flavor"]["name"]
            .as_str()
            .unwrap()
            .to_string(),
        language_code: raw_metadata_struct["languages"][0]["tag"]
            .as_str()
            .unwrap()
            .to_string(),
        script_direction: match raw_metadata_struct["languages"][0]["scriptDirection"].clone() {
            Value::String(v) => v.as_str().to_string(),
            _ => "?".to_string(),
        },
        book_codes: book_codes,
        timestamp: std::fs::metadata(&repo_metadata_path)
            .expect("Could not read fs metadata")
            .modified()
            .expect("Could not get modified for fs")
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("Could not get elapsed")
            .as_secs(),
    })
}

pub(crate) fn destination_parent(destination: String) -> String {
    let mut destination_steps: Vec<_> = destination.split("/").collect();
    destination_steps.pop().unwrap();
    let destination_steps_array = destination_steps
        .iter()
        .map(|e| format!("{:?}", e).replace("\"", ""))
        .collect::<Vec<String>>();
    destination_steps_array.join("/")
}
