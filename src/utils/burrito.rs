use std::io;
use serde_json::Value;
use crate::structs::MetadataSummary;

pub(crate) fn summary_metadata_from_file(repo_metadata_path: String) -> Result<MetadataSummary, io::Error> {
    let file_string = match std::fs::read_to_string(repo_metadata_path) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let raw_metadata_struct: Value = match serde_json::from_str(file_string.as_str()) {
        Ok(v) => v,
        Err(e) => {
            return Err(io::Error::from(e));
        }
    };
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
    })
}
