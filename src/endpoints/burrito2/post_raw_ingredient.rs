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
    let full_repo_path = state.repo_dir.lock().unwrap().clone()
        + os_slash_str()
        + &repo_path.display().to_string();

    if check_path_components(&mut path_components.clone())
        && check_path_string_components(ipath.clone())
        && std::fs::metadata(&full_repo_path).is_ok()
    {
        let destination = state.repo_dir.lock().unwrap().clone()
            + os_slash_str()
            + &repo_path.display().to_string()
            + "/ingredients/"
            + ipath.clone().as_str();
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
