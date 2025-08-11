use crate::structs::AppSettings;
use crate::utils::json_responses::{make_bad_json_data_response};
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, json_payload_response,
};
use git2::Repository;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};
use serde_json::{json, Value};
use std::path::{Components, PathBuf};

/// *`GET /remotes/<repo_path>`*
///
/// Typically mounted as **`/git/remotes/<repo_path>`**
///
/// List remotes for the given repo path.
#[get("/remotes/<repo_path..>")]
pub async fn list_remotes_for_repo(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let repo_path_string = format!(
            "{}{}{}",
            state.repo_dir.lock().unwrap().clone(),
            os_slash_str(),
            &repo_path.display().to_string()
        );
        match Repository::open(repo_path_string) {
            Ok(repo) => {
                let remotes = match repo.remotes() {
                    Ok(v) => v,
                    Err(e) => {
                        return not_ok_json_response(
                            Status::InternalServerError,
                            make_bad_json_data_response(format!(
                                "Could not list remotes for repo: {}",
                                e
                            )),
                        )
                    }
                };
                let remote_vec: Vec<Value> = remotes
                    .iter()
                    .map(
                        |r| -> Value {
                            let remote = repo.find_remote(r.unwrap()).unwrap();
                            return json!({"name": remote.name(), "url": remote.url()});
                        }
                    )
                    .collect();
                let return_json = json!({ "remotes": remote_vec });
                json_payload_response(
                    Status::Ok,
                    return_json,
                )
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
