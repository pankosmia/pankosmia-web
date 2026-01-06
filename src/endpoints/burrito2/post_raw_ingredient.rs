use crate::structs::{AppSettings, BurritoMetadata};
use crate::utils::json_responses::{make_bad_json_data_response};
use crate::utils::paths::{check_path_components, check_path_string_components, os_slash_str};
use crate::utils::response::{not_ok_json_response, ok_ok_json_response, not_ok_bad_repo_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, State};
use serde_json::Value;
use std::path::{Components, PathBuf};
use crate::utils::burrito::{destination_parent, ingredients_metadata_from_files, ingredients_scopes_from_files};

/// *`POST /ingredient/raw/<repo_path>?ipath=my_burrito_path&update_ingredients`*
///
/// Typically mounted as **`/burrito/ingredient/raw/<repo_path>?ipath=my_burrito_path`**
///
/// Writes a document, where the document is provided as JSON with a 'payload' key.

#[post(
    "/ingredient/raw/<repo_path..>?<ipath>&<update_ingredients>",
    format = "json",
    data = "<json_form>"
)]
#[allow(irrefutable_let_patterns)]
pub async fn post_raw_ingredient(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    ipath: String,
    update_ingredients: Option<String>,
    json_form: Json<Value>,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    let repo_dir = state.repo_dir.lock().expect("lock for repo dir");
    let full_repo_path =
        format!("{}{}{}", &repo_dir, os_slash_str(), &repo_path.display().to_string());
    if check_path_components(&mut path_components.clone())
        && check_path_string_components(ipath.clone())
        && std::fs::metadata(&full_repo_path).is_ok()
    {
        let destination = format!("{}{}ingredients{}{}", &full_repo_path, os_slash_str(), os_slash_str(), &ipath);
        let destination_parent = destination_parent(destination.clone());
        // Make subdirs if necessary
        if !std::path::Path::new(&destination_parent).exists() {
            match std::fs::create_dir_all(destination_parent) {
                Ok(_) => (),
                Err(e) => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!(
                            "Could not create local content directories: {}",
                            e
                        )),
                    )
                }
            }
        }
        // Maybe make backup file
        let destination_backup_path = format!("{}.bak", &destination);
        println!("{}, {}", &destination, &destination_backup_path);
        if std::path::Path::new(&destination).exists() {
            match std::fs::rename(&destination, &destination_backup_path) {
                Ok(_) => (),
                Err(e) => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!("Could not write backup file: {}", e)),
                    )
                }
            }
        }
        match std::fs::write(destination, json_form["payload"].as_str().unwrap()) {
            Ok(_) => {},
            Err(e) => return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not write to {}: {}", ipath, e)),
            ),
        };
        if update_ingredients.is_some() {
            // Get metadata as struct
            let app_resources_dir = format!("{}", &state.app_resources_dir);
            let path_to_repo_metadata = format!(
                "{}{}metadata.json",
                &full_repo_path,
                os_slash_str(),
            );
            let metadata_string = match std::fs::read_to_string(&path_to_repo_metadata) {
                Ok(v) => v,
                Err(e) => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!(
                            "Could not load metadata as string: {}",
                            e
                        )),
                    )
                }
            };
            // Make struct from metadata
            let mut metadata_struct: BurritoMetadata = match serde_json::from_str(&metadata_string) {
                Ok(v) => v,
                Err(e) => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!("Could not parse metadata: {}", e)),
                    );
                }
            };
            // Add ingredient record and currentScope value for USFM
            if let mut ingredients = metadata_struct.ingredients.lock().unwrap() {
                let new_ingredients = ingredients_metadata_from_files(app_resources_dir.clone(), full_repo_path.clone());
                *ingredients = new_ingredients;
            }
            if let type_info = metadata_struct.r#type {
                let mut type_ob = type_info.as_object().unwrap().clone();
                let flavor_type_ob = type_ob["flavorType"].as_object_mut().unwrap();
                let new_current_scope = ingredients_scopes_from_files(app_resources_dir, full_repo_path.clone());
                flavor_type_ob["currentScope"] = serde_json::from_str(serde_json::to_string(&new_current_scope).unwrap().as_str()).unwrap();
                metadata_struct.r#type = serde_json::from_str(serde_json::to_string(&type_ob).unwrap().as_str()).unwrap();
            }

            // Write metadata
            let metadata_output_string = match serde_json::to_string(&metadata_struct) {
                Ok(s) => s,
                Err(e) => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!("Could not make metadata as JSON: {}", e)),
                    )
                }
            };
            match std::fs::write(path_to_repo_metadata, &metadata_output_string) {
                Ok(_) => (),
                Err(e) => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!("Could not write metadata to repo: {}", e)),
                    )
                }
            }
        }
        ok_ok_json_response()
    } else {
        not_ok_bad_repo_json_response()
    }
}
