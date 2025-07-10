use crate::structs::AppSettings;
use crate::utils::json_responses::{make_bad_json_data_response};
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{not_ok_json_response, not_ok_bad_repo_json_response, ok_ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{post, State};
use std::path::{Components, PathBuf};

/// *`POST /delete/<repo_path>`*
///
/// Typically mounted as **`/git/delete/<repo_path>`**
///
/// Deletes a local repo from the given repo path.
#[post("/delete/<repo_path..>")]
pub async fn delete_repo(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let path_to_delete = state.repo_dir.lock().unwrap().clone()
            + os_slash_str()
            + &repo_path.display().to_string();
        match std::fs::remove_dir_all(path_to_delete) {
            Ok(_) => ok_ok_json_response(),
            Err(e) => not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!("could not delete repo: {}", e).to_string()),
            ),
        }
    } else {
        not_ok_bad_repo_json_response()
    }
}
