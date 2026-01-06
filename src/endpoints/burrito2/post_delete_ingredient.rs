use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, check_path_string_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_ok_json_response,
};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{post, State};
use std::path::{Components, PathBuf};

/// *`POST /ingredient/delete/<repo_path>?ipath=my_burrito_path`*
///
/// Typically mounted as **`/burrito/ingredient/delete/<repo_path>?ipath=my_burrito_path`**
///
/// Deletes a file from a repo.
#[post("/ingredient/delete/<repo_path..>?<ipath>")]
pub async fn post_delete_ingredient(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    ipath: String,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    let repo_dir = state.repo_dir.lock().expect("lock for repo dir");
    let full_repo_path =
        repo_dir.clone() + os_slash_str() + &repo_path.display().to_string();
    if check_path_components(&mut path_components.clone())
        && check_path_string_components(ipath.clone())
        && std::fs::metadata(&full_repo_path).is_ok()
    {
        let destination = full_repo_path + "/ingredients/" + ipath.clone().as_str();
        if !std::path::Path::new(&destination).exists() {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!("No such file: {}", destination)),
            );
        }
        let destination_backup_path = format!("{}.bak", &destination);
        match std::fs::rename(&destination, &destination_backup_path) {
            Ok(_) => ok_ok_json_response(),
            Err(e) => {
                not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Could not delete file: {}", e)),
                )
            }
        }
    } else {
        not_ok_bad_repo_json_response()
    }
}
