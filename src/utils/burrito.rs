use std::collections::BTreeMap;
use crate::structs::{BurritoMetadataIngredient, MetadataSummary};
use serde_json::{json, Map, Value};
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use walkdir::WalkDir;
use crate::utils::paths::os_slash_str;
use regex::Regex;
use chksum_md5::chksum;
use mime_infer;

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

pub fn ingredients_metadata_from_files(
    repo_path: String,
) -> BTreeMap<String, BurritoMetadataIngredient> {
        let mut ingredients = BTreeMap::new();
        for entry in WalkDir::new(&repo_path) {
            let entry_string = entry.unwrap().path().display().to_string();
            if Path::new(&entry_string).is_file() {
                let truncated_entry_string = entry_string.replace(&repo_path, "");
                if !truncated_entry_string.starts_with(".") && !truncated_entry_string.contains(format!("{}.", os_slash_str()).as_str()) {
                    let mut ingredient_scope: Option<Value> = None;
                    let entry_copy = truncated_entry_string.clone();
                    let file_path_parts: Vec<_> = entry_copy.split("/").collect();
                    let file_name_parts: Vec<_> = file_path_parts.last().unwrap().split(".").collect();
                    if file_name_parts.len() < 2 {
                        continue;
                    }
                    if file_name_parts[0] == "metadata" && file_name_parts[1] == "json" {
                        continue;
                    }
                    if file_name_parts.len() == 3 && file_name_parts[1] == "bak" {
                        continue;
                    }
                    let file_part1 = file_name_parts[0];
                    // Scope
                    let bible_regex = Regex::new("^[1-6A-Z]{3}$").unwrap();
                    if bible_regex.is_match(&file_part1) {
                        ingredient_scope = Some(json!({file_part1.to_string(): []}));
                    }
                    // Size
                    let ingredient_size = fs::metadata(&entry_string).unwrap().len();
                    // md5
                    let chk_file = File::open(&entry_string).unwrap();
                    let ingredient_md5 = chksum(chk_file).unwrap().to_string();
                    // mimeType
                    let ingredient_mime_type = match mime_infer::from_path(&entry_string).first() {
                        Some(mime_type) => mime_type.to_string(),
                        None => {
                            if file_name_parts.len() == 2 && (file_name_parts[1] == "usfm" || file_name_parts[1] == "vrs") {
                                "text/plain".to_string()
                            } else {
                                "application/octet-stream".to_string()
                            }
                        },
                    };
                    let ingredient_details = BurritoMetadataIngredient {
                        checksum: json!({"md5": ingredient_md5}),
                        mimeType: ingredient_mime_type.to_string(),
                        size: ingredient_size as usize,
                        scope: ingredient_scope
                    };
                    ingredients.insert(
                        truncated_entry_string
                            .replace("\\", "/")
                            .replace("/ingredients/", "ingredients/"),
                        ingredient_details
                    );
                }
            }
        }
    ingredients
}

pub fn ingredients_scopes_from_files(repo_path: String) -> BTreeMap<String, Value> {
    let mut scopes = BTreeMap::new();
    let ingredients_map = ingredients_metadata_from_files(repo_path);
    for (_key, value) in ingredients_map.iter() {
        match value.clone().scope {
            None => {},
            Some(_) => {
                let scope_object_value = value.clone().scope.unwrap();
                let scope_object = scope_object_value.as_object().unwrap();
                for (jk, jv) in scope_object {
                    scopes.insert(jk.to_string(), jv.clone());
                }
            }
        }
    }
    scopes
}