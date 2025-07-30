use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, check_path_string_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_ok_json_response,
};
use copy_dir::copy_dir;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{post, State};
use std::path::{Components, PathBuf};
use crate::utils::burrito::destination_parent;

/// *`POST /copy/<repo_path>?target_path=<target_path>&delete_src`*
///
/// Typically mounted as **`/git/copy/<repo_path>?target_path=<target_path>&delete_src`**
///
/// Copies a repo to a new location, optionally deleting the source.
#[post("/copy/<repo_path..>?<target_path>&<delete_src>")]
pub async fn copy_repo(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    target_path: String,
    delete_src: Option<bool>,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone())
        && check_path_string_components(target_path.clone())
    {
        let full_src_path = format!(
            "{}{}{}",
            &state.repo_dir.lock().unwrap(),
            os_slash_str(),
            &repo_path.display().to_string()
        );
        // Src repo must exist
        if !std::path::Path::new(&full_src_path).is_dir() {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("Source repo not found or not a directory".to_string()),
            );
        }
        let full_target_path = format!(
            "{}{}{}",
            &state.repo_dir.lock().unwrap(),
            os_slash_str(),
            &target_path
        );
        // Target repo dir must not exist
        if std::path::Path::new(&full_target_path).is_dir() {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("Target repo already exists".to_string()),
            );
        }
        // Make subdirs if necessary
        let target_parent = destination_parent(full_target_path.clone());
        if !std::path::Path::new(&target_parent).exists() {
            match std::fs::create_dir_all(target_parent) {
                Ok(_) => (),
                Err(e) => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!(
                            "Could not create target parent directories: {}",
                            e
                        )),
                    )
                }
            }
        }
        // copy repo
        match copy_dir(full_src_path.clone(), full_target_path) {
            Ok(_) => {}
            Err(e) => {
                return not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(format!("could not copy repo: {}", e).to_string()),
                )
            }
        }
        // Maybe delete src repo
        match delete_src {
            Some(true) => {
                match std::fs::remove_dir_all(full_src_path) {
                    Ok(_) => { },
                    Err(e) => return not_ok_json_response(
                        Status::BadRequest,
                        make_bad_json_data_response(format!("could not delete src repo: {}", e).to_string()),
                    ),
                }
            },
            _ => {}
        }
        ok_ok_json_response()
    } else {
        not_ok_bad_repo_json_response()
    }
}
