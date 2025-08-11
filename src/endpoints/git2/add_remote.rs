use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_ok_json_response,
};
use git2::Repository;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{post, State};
use std::path::{Components, PathBuf};
use regex::Regex;

/// *`POST /remote/add/<repo_path>?remote=<remote_name>&url=<remote_url>`*
///
/// Typically mounted as **`/git/remote/add/<repo_path>?remote=<remote_name>&url=<remote_url>`**
///
/// Adds a remote to the given repo path.
#[post("/remote/add/<repo_path..>?<remote_name>&<remote_url>")]
pub async fn add_remote_to_repo(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    remote_name: String,
    remote_url: String,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let repo_path_string = format!(
            "{}{}{}", 
            state.repo_dir.lock().unwrap().clone(),
            os_slash_str(),
            &repo_path.display().to_string()
        );
        let remote_name_re = Regex::new(r"^[A-Za-z0-9_-]$").unwrap();
        if !remote_name_re.is_match(&remote_name) {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("Remote name contains invalid characters".to_string()),
            )
        }
        let remote_url_re = Regex::new(r"^[A-Za-z0-9_:/-]$").unwrap();
        if !remote_url_re.is_match(&remote_name) {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("Remote url contains invalid characters".to_string()),
            )
        }
        if !remote_url.starts_with("https://") {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("Remote URL does not use HTTPS".to_string()),
            )
        }
        match Repository::open(repo_path_string) {
            Ok(repo) => {
                match repo.remote(&remote_name, &remote_url) {
                    Ok(_) => ok_ok_json_response(),
                    Err(e) => {
                        not_ok_json_response(
                            Status::InternalServerError,
                            make_bad_json_data_response(format!("Could not add remote to repo: {}", e))
                        )
                    }
                }
            }
            Err(e) => not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not open repo: {}", e))
            ),
        }
    } else {
        not_ok_bad_repo_json_response()
    }
}
