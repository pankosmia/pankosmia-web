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
use crate::utils::burrito::destination_parent;

/// *`POST /ingredient/copy/<repo_path>?src_path=<src_path>&target_path=<target_path>&delete_src`*
///
/// Typically mounted as **`/burrito/copy/<repo_path>?src_path=<src_path>&target_path=<target_path>&delete_src`**
///
/// Copies an ingredient to a new location, optionally deleting the source.
#[post("/ingredient/copy/<repo_path..>?<src_path>&<target_path>&<delete_src>")]
pub async fn copy_ingredient(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    src_path: String,
    target_path: String,
    delete_src: Option<bool>,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone())
        && check_path_string_components(src_path.clone())
        && check_path_string_components(target_path.clone())
    {
        let full_src_path = format!(
            "{}{}{}{}ingredients{}{}",
            &state.repo_dir.lock().unwrap(),
            os_slash_str(),
            &repo_path.display().to_string(),
            os_slash_str(),
            os_slash_str(),
            &src_path
        );
        // Src ingredient must exist
        if !std::path::Path::new(&full_src_path).is_file() {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("Source ingredient not found or not a file".to_string()),
            );
        }
        let full_target_path = format!(
            "{}{}{}{}ingredients{}{}",
            &state.repo_dir.lock().unwrap(),
            os_slash_str(),
            &repo_path.display().to_string(),
            os_slash_str(),
            os_slash_str(),
            &target_path
        );
        // src and target must not be identical
        if full_src_path == full_target_path {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("src and target must be different".to_string()),
            )
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
        // Maybe make backup file
        let destination_backup_path = format!("{}.bak", &full_target_path);
        if std::path::Path::new(&full_target_path).exists() {
            match std::fs::rename(&full_target_path, &destination_backup_path) {
                Ok(_) => (),
                Err(e) => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!("Could not write backup file: {}", e)),
                    )
                }
            }
        }
        // copy ingredient
        match std::fs::copy(full_src_path.clone(), full_target_path) {
            Ok(_) => {}
            Err(e) => {
                return not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(format!("could not copy ingredient: {}", e).to_string()),
                )
            }
        }
        // Maybe delete src ingredient
        match delete_src {
            Some(true) => {
                match std::fs::remove_file(full_src_path) {
                    Ok(_) => { },
                    Err(e) => return not_ok_json_response(
                        Status::BadRequest,
                        make_bad_json_data_response(format!("could not delete src ingredient: {}", e).to_string()),
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
