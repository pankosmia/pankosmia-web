use crate::structs::AppSettings;
use crate::utils::json_responses::{make_bad_json_data_response};
use crate::utils::paths::{check_path_components, check_path_string_components, os_slash_str};
use crate::utils::response::{not_ok_json_response, ok_ok_json_response, not_ok_bad_repo_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, State};
use serde_json::Value;
use std::path::{Components, PathBuf};
use crate::utils::burrito::destination_parent;

/// *`POST /ingredient/raw/<repo_path>?ipath=my_burrito_path`*
///
/// Typically mounted as **`/burrito/ingredient/raw/<repo_path>?ipath=my_burrito_path`**
///
/// Writes a document, where the document is provided as JSON with a 'payload' key.
///
/// /// The target file must exist, ie this is not the way to add new ingredients to a Burrito
#[post(
    "/ingredient/raw/<repo_path..>?<ipath>",
    format = "json",
    data = "<json_form>"
)]
pub async fn post_raw_ingredient(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    ipath: String,
    json_form: Json<Value>,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    let full_repo_path =
        format!("{}{}{}", state.repo_dir.lock().unwrap(), os_slash_str(), &repo_path.display().to_string());
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
        ok_ok_json_response()
    } else {
        not_ok_bad_repo_json_response()
    }
}
