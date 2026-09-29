use crate::structs::{BurritoMetadataIngredient, MetadataSummary, PankosmiaError};
use crate::utils::bcv_ref::canonical_book_codes;
use crate::utils::files::load_json;
use crate::utils::paths::os_slash_str;
use chksum_md5::chksum;
use mime_infer;
use regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use walkdir::WalkDir;

pub(crate) fn copy_vrs_template(
    app_resources_dir: &String,
    repo_dir: &String,
    versification: &String,
) -> Result<Value, PankosmiaError> {
    // Get versification file as JSON
    let path_to_versification = format!(
        "{}{}templates{}content_templates{}vrs{}{}.json",
        app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        &versification,
    );
    let versification_schema = match load_json(&path_to_versification) {
        Ok(j) => j,
        Err(e) => {
            return Err(PankosmiaError(format!(
                "Could not load versification JSON: {}",
                e
            )))
        }
    };
    // Write it out to new repo
    let path_to_repo_versification = format!("{}{}ingredients/vrs.json", repo_dir, os_slash_str(),);
    let versification_string = serde_json::to_string(&versification_schema).unwrap();
    match std::fs::write(path_to_repo_versification, &versification_string) {
        Ok(_) => Ok(versification_schema),
        Err(e) => Err(PankosmiaError(format!(
            "Could not write versification to repo: {}",
            e
        ))),
    }
}

pub(crate) fn copy_gitignore_template(
    app_resources_dir: &String,
    repo_dir: &String,
) -> Result<(), PankosmiaError> {
    // Copy gitignore file
    let path_to_gitignore_template = format!(
        "{}{}templates{}content_templates{}gitignore.txt",
        app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
    );
    let gitignore_string = match std::fs::read_to_string(&path_to_gitignore_template) {
        Ok(v) => v,
        Err(e) => {
            return Err(PankosmiaError(format!(
                "Could not load gitignore template as string: {}",
                e
            )))
        }
    };
    let path_to_repo_gitignore = format!("{}{}.gitignore", repo_dir, os_slash_str(),);
    match std::fs::write(path_to_repo_gitignore, &gitignore_string) {
        Ok(_) => Ok(()),
        Err(e) => Err(PankosmiaError(format!(
            "Could not write gitignore to repo: {}",
            e
        ))),
    }
}

pub(crate) fn language_name_from_code(
    app_resources_dir: &String,
    language_code: String,
    supplied_language_name: Option<String>,
) -> Result<String, PankosmiaError> {
    // Custom language begins with x- and name must be provided
    // Non-custom language must be in lookup, provided name is ignored
    // First regex validate the bcp47 string
    let bcp_regex = Regex::new("^(((en-GB-oed|i-ami|i-bnn|i-default|i-enochian|i-hak|i-klingon|i-lux|i-mingo|i-navajo|i-pwn|i-tao|i-tay|i-tsu|sgn-BE-FR|sgn-BE-NL|sgn-CH-DE)|(art-lojban|cel-gaulish|no-bok|no-nyn|zh-guoyu|zh-hakka|zh-min|zh-min-nan|zh-xiang))|((([A-Za-z]{2,3}(-([A-Za-z]{3}(-[A-Za-z]{3}){0,2}))?)|[A-Za-z]{4}|[A-Za-z]{5,8})(-([A-Za-z]{4}))?(-([A-Za-z]{2}|[0-9]{3}))?(-([A-Za-z0-9]{5,8}|[0-9][A-Za-z0-9]{3}))*(-([0-9A-WY-Za-wy-z](-[A-Za-z0-9]{2,8})+))*(-(x(-[A-Za-z0-9]{1,8})+))?)|(x(-[A-Za-z0-9]{1,8})+))$").unwrap();
    if !bcp_regex.is_match(&language_code) {
        return Err(PankosmiaError(format!(
            "Language code '{}' is not Scripture Burrito schema valid",
            &language_code
        )));
    }
    // To x- or not to x-
    if language_code.starts_with("x-") {
        match supplied_language_name {
            Some(n) => Ok(n),
            None => Err(PankosmiaError(format!(
                "Language code '{}' is custom ('x-') but no language name has been provided",
                &language_code
            ))),
        }
    } else {
        // Read language lookup
        let path_to_language_lookup = format!(
            "{}{}app_resources{}lookups{}bcp47-language_codes.json",
            app_resources_dir,
            os_slash_str(),
            os_slash_str(),
            os_slash_str(),
        );

        let language_lookup_json = match load_json(&path_to_language_lookup) {
            Ok(v) => v,
            Err(e) => {
                return Err(PankosmiaError(format!(
                    "Could not load and parse language lookup: {}",
                    e
                )));
            }
        };

        // Split non-x- name on first dash to get iso 639-[13] code for lookup
        let mut language_code_bits = language_code
            .split("-")
            .collect::<std::collections::VecDeque<&str>>();
        // Attempt to lookup that language code
        let mut language_name = match language_lookup_json[language_code_bits[0]].as_object() {
            Some(r) => r["en"].as_str().expect("English language name").to_string(),
            None => {return Err(PankosmiaError(format!(
                    "Language code '{}' is not custom (no 'x-') but has not been found in the BCP47 lookup table",
                    &language_code
                )))}
        };
        // Add any bcp47 qualifiers to name if necessary
        if language_code_bits.len() > 1 {
            language_code_bits.pop_front();
            let language_code_bits_vec = Into::<Vec<&str>>::into(language_code_bits);
            language_name = format!("{} ({})", &language_name, language_code_bits_vec.join(" "));
        }
        Ok(language_name)
    }
}

pub(crate) fn summary_metadata_from_file(
    repo_metadata_path: String,
) -> Result<MetadataSummary, io::Error> {
    let file_string = match fs::read_to_string(&repo_metadata_path) {
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
        language_name: raw_metadata_struct["languages"][0]["name"]["en"]
            .as_str()
            .unwrap()
            .to_string(),
        script_direction: match raw_metadata_struct["languages"][0]["scriptDirection"].clone() {
            Value::String(v) => v.as_str().to_string(),
            _ => "?".to_string(),
        },
        book_codes: book_codes,
        timestamp: raw_metadata_struct["identification"]["primary"]
            .as_object()
            .and_then(|primary| {
                primary.values().find_map(|value| {
                    value.as_object().and_then(|inner| {
                        inner.values().find_map(|value| value["timestamp"].as_str())
                    })
                })
            })
            .and_then(|timestamp| {
                let dt = chrono::DateTime::parse_from_rfc3339(timestamp).ok()?;
                std::time::SystemTime::from(dt)
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .ok()
                    .map(|elapsed| elapsed.as_secs())
            })
            .unwrap_or_else(|| {
                fs::metadata(&repo_metadata_path)
                    .expect("Could not read fs metadata")
                    .modified()
                    .expect("Could not get modified for fs")
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .expect("Could not get elapsed")
                    .as_secs()
            }),
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
    app_resources_dir: String,
    repo_path: String,
) -> BTreeMap<String, BurritoMetadataIngredient> {
    let mut ingredients = BTreeMap::new();
    for entry in WalkDir::new(&repo_path) {
        let entry_string = entry.unwrap().path().display().to_string();
        if Path::new(&entry_string).is_file() {
            let truncated_entry_string = entry_string.replace(&repo_path, "");
            if !truncated_entry_string.starts_with(".")
                && !truncated_entry_string.contains(format!("{}.", os_slash_str()).as_str())
            {
                let mut ingredient_scope: Option<Value> = None;
                let entry_copy = truncated_entry_string.clone();
                let file_path_parts: Vec<_> = entry_copy.split(os_slash_str()).collect();
                let file_name_parts: Vec<_> = file_path_parts.last().unwrap().split(".").collect();
                if file_name_parts.len() < 2 {
                    continue;
                }
                if file_name_parts[0] == "metadata" && file_name_parts[1] == "json" {
                    continue;
                }
                if file_name_parts.len() == 3 && file_name_parts[2] == "bak" {
                    continue;
                }
                let file_part1 = file_name_parts[0];
                // Scope
                let bible_regex = Regex::new("^[1-6A-Z]{3}$").unwrap();
                let book_string = file_part1.to_string();
                if bible_regex.is_match(&file_part1)
                    && canonical_book_codes(app_resources_dir.clone()).contains(&book_string)
                {
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
                        if file_name_parts.len() == 2
                            && (file_name_parts[1] == "usfm" || file_name_parts[1] == "vrs")
                        {
                            "text/plain".to_string()
                        } else {
                            "application/octet-stream".to_string()
                        }
                    }
                };
                let ingredient_details = BurritoMetadataIngredient {
                    checksum: json!({"md5": ingredient_md5}),
                    mimeType: ingredient_mime_type.to_string(),
                    size: ingredient_size as usize,
                    scope: ingredient_scope,
                    role: None,
                };
                ingredients.insert(
                    truncated_entry_string
                        .replace("\\", "/")
                        .replace("/ingredients/", "ingredients/"),
                    ingredient_details,
                );
            }
        }
    }
    ingredients
}

pub fn ingredients_scopes_from_files(
    app_resources_dir: String,
    repo_path: String,
) -> BTreeMap<String, Value> {
    let mut scopes = BTreeMap::new();
    let ingredients_map = ingredients_metadata_from_files(app_resources_dir, repo_path);
    for (_key, value) in ingredients_map.iter() {
        match value.clone().scope {
            None => {}
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
