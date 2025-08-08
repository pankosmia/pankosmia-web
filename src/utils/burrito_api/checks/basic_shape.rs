use serde_json::Value;
use crate::utils::burrito_api::checks::report_helpers::{CheckReport, ok_check_report};

pub(crate) fn check_basic_shape(burrito_path: String) -> Vec<CheckReport> {
    // Top-level directory
    let mut reports = vec![];
    let burrito_path_path = std::path::Path::new(&burrito_path);
    if !&burrito_path_path.exists() {
        reports.push(CheckReport {
            name: "BurritoShape:Container:Exists".to_string(),
            path: burrito_path.clone(),
            success: false,
            comment: Some("Burrito path not found".to_string()),
            data: None,
        });
        return reports;
    } else if !&burrito_path_path.is_dir() {
        reports.push(CheckReport {
            name: "BurritoShape:Container:IsDir".to_string(),
            path: burrito_path.clone(),
            success: false,
            comment: Some("Burrito path exists but is not a directory".to_string()),
            data: None,
        });
        return reports;
    } else {
        reports.push(ok_check_report(
            "BurritoShape:Container".to_string(),
            burrito_path.clone(),
        ))
    }

    // Metadata exists and is JSON
    let metadata_path = format!("{}/metadata.json", burrito_path);
    let metadata_path_path = std::path::Path::new(&metadata_path);
    if !&metadata_path_path.exists() {
        reports.push(CheckReport {
            name: "BurritoShape:Metadata:Exists".to_string(),
            path: burrito_path.clone(),
            success: false,
            comment: Some("Metadata not found".to_string()),
            data: None,
        })
    } else if !&metadata_path_path.is_file() {
        reports.push(CheckReport {
            name: "BurritoShape:Metadata:IsFile".to_string(),
            path: burrito_path.clone(),
            success: false,
            comment: Some("Metadata exists but is not a file".to_string()),
            data: None,
        })
    }
    match std::fs::read_to_string(metadata_path) {
        Ok(metadata_string) => match serde_json::from_str::<Value>(&metadata_string) {
            Ok(_) => {
                reports.push(ok_check_report(
                    "BurritoShape:Metadata".to_string(),
                    burrito_path.clone(),
                ))
            }
            Err(e) => {
                reports.push(CheckReport {
                    name: "BurritoShape:Metadata:IsJson".to_string(),
                    path: burrito_path.clone(),
                    success: false,
                    comment: Some("Metadata exists but cannot be parsed as JSON".to_string()),
                    data: Some(vec![e.to_string()]),
                })
            }
        },
        Err(e) => reports.push(CheckReport {
            name: "BurritoShape:Metadata:IsReadable".to_string(),
            path: burrito_path.clone(),
            success: false,
            comment: Some("Metadata exists but cannot be read".to_string()),
            data: Some(vec![e.to_string()]),
        }),
    };
    // Ingredients exists and is directory
    let ingredients_path = format!("{}/ingredients", burrito_path);
    let ingredients_path_path = std::path::Path::new(&ingredients_path);
    if !&ingredients_path_path.exists() {
        reports.push(CheckReport {
            name: "BurritoShape:Ingredients:Exists".to_string(),
            path: burrito_path.clone(),
            success: false,
            comment: Some("Ingredients dir not found".to_string()),
            data: None,
        })
    } else if !&ingredients_path_path.is_dir() {
        reports.push(CheckReport {
            name: "BurritoShape:Ingredients:IsDir".to_string(),
            path: burrito_path.clone(),
            success: false,
            comment: Some("Ingredients exists but is not a directory".to_string()),
            data: None,
        })
    } else {
        reports.push(ok_check_report(
            "BurritoShape:Ingredients".to_string(),
            burrito_path.clone(),
        ))
    }
    // No unexpected top-level content
    match std::fs::read_dir(&burrito_path) {
        Ok(burrito_iter) => {
            let mut unexpected = vec![];
            for wrapped_entry in burrito_iter {
                let entry_string = match wrapped_entry {
                    Ok(entry) => {
                        let v = entry.file_name();
                        let v2 =  v.to_str().unwrap();
                        v2.to_string()
                    },
                    Err(e) => {
                        reports.push(CheckReport {
                            name: "BurritoShape:Content:ContentListable".to_string(),
                            path: burrito_path.clone(),
                            success: false,
                            comment: Some("Some content cannot be listed".to_string()),
                            data: Some(vec![e.to_string()]),
                        });
                        break;
                    }
                };
                if entry_string != "metadata.json" && entry_string != "ingredients" && !entry_string.starts_with(".") {
                    unexpected.push(entry_string);
                }
            }
            if !unexpected.is_empty() {
                reports.push(CheckReport {
                    name: "BurritoShape:Content:Unexpected".to_string(),
                    path: burrito_path.clone(),
                    success: false,
                    comment: Some("Unexpected content".to_string()),
                    data: Some(unexpected),
                });
            }
        },
        Err(e) => {
            reports.push(CheckReport {
                name: "BurritoShape:Ingredients:IsReadable".to_string(),
                path: burrito_path.clone(),
                success: false,
                comment: Some("Ingredients exists but content cannot be listed".to_string()),
                data: Some(vec![e.to_string()]),
            })
        }
    }
    reports
}
