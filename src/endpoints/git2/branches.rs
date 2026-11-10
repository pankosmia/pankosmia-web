use crate::structs::AppSettings;
use crate::utils::git::git_repo_path;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::check_path_components;
use crate::utils::response::{
    json_payload_response, not_ok_bad_repo_json_response, not_ok_json_response,
};
use git2::Repository;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};
use serde_json::{json, Value};
use std::path::{Components, PathBuf};

/// *`GET /branches/<repo_path>`*
///
/// Typically mounted as **`/git/branches/<repo_path>`**
///
/// List branches for the given repo path.
#[get("/branches/<repo_path..>")]
pub async fn list_branches_for_repo(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let repo_dir_string = state.repo_dir.lock().unwrap().clone();
        let repo_url_string = repo_path.display().to_string();
        let repo_path_string = match git_repo_path(repo_dir_string.clone(), repo_url_string.clone())
        {
            Some(s) => s,
            None => {
                return not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(format!(
                        "repo directory '{}' does not exist or does not contain a .git directory",
                        &repo_url_string
                    )),
                );
            }
        };
        match Repository::open(repo_path_string) {
            Ok(repo) => {
                let branches = match repo.branches(None) {
                    Ok(v) => v,
                    Err(e) => {
                        return not_ok_json_response(
                            Status::InternalServerError,
                            make_bad_json_data_response(format!(
                                "Could not list branches for repo: {}",
                                e
                            )),
                        )
                    }
                };
                let head = repo.head().expect("Could not locate head");
                let head_branch_name = head.name().expect("Could not get branch name from head");
                let branch_vec: Vec<Value> = branches
                    .map(|b| -> Value {
                        match b {
                            Ok(v) => {
                                let bname = v.0.name().unwrap().unwrap_or("??");
                                let is_head =
                                    head_branch_name.ends_with(format!("/{}", bname).as_str());
                                json!({"name": bname, "is_head": is_head })
                            }
                            Err(_) => json!({"name": "???"}),
                        }
                    })
                    .collect();
                let return_json = json!({ "branches": branch_vec });
                json_payload_response(Status::Ok, return_json)
            }
            Err(e) => not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not open repo: {}", e)),
            ),
        }
    } else {
        not_ok_bad_repo_json_response()
    }
}
