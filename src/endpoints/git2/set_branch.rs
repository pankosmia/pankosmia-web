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

/// *`POST /branch/<branch_ref>/<repo_path>`*
///
/// Typically mounted as **`/git/branch/<branch_ref>/<repo_path>`**
///
/// Changes the selected branch for a given repo. The branch must exist.

#[post("/branch/<branch_ref>/<repo_path..>")]
pub async fn set_branch(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    branch_ref: String,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let repo_path_string = format!(
            "{}{}{}",
            state.repo_dir.lock().unwrap().clone(),
            os_slash_str(),
            &repo_path.display().to_string().clone()
        );
        match Repository::open(repo_path_string) {
            Ok(repo) => {
                let branch_full_name = format!("refs/heads/{}", branch_ref);
                match repo.set_head(&branch_full_name) {
                    Ok(_) => ok_ok_json_response(),
                    Err(e) => {
                        not_ok_json_response(
                            Status::InternalServerError,
                            make_bad_json_data_response(
                                format!("could not set head: {}", e).to_string(),
                            ),
                        )
                    }
                }
            }
            Err(e) => not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(
                    format!("could open repo: {}", e).to_string(),
                ),
            ),
        }
    } else {
        not_ok_bad_repo_json_response()
    }
}
