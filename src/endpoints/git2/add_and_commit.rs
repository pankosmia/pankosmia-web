use std::path::PathBuf;
use git2::Repository;
use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::structs::AppSettings;
use crate::utils::json_responses::{make_bad_json_data_response, make_good_json_data_response};
use crate::utils::paths::os_slash_str;

/// *`GET /add-and-commit/<repo_path>`*
///
/// Typically mounted as **`/git/add-and-commit/<repo_path>`**
///
/// Adds and commits modified files for a given repo.
#[get("/add-and-commit/<repo_path..>")]
pub async fn add_and_commit(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let repo_path_string: String = state.repo_dir.lock().unwrap().clone()
        + os_slash_str()
        + &repo_path.display().to_string().clone();
    let result = match Repository::open(repo_path_string) {
        Ok(repo) => {
            repo.index()
                .unwrap()
                .add_all(&["."], git2::IndexAddOption::DEFAULT, None)
                .unwrap();
            repo.index().unwrap().write().unwrap();
            let mut index = repo.index().unwrap();
            let oid = index.write_tree().unwrap();
            let signature = repo.signature().unwrap();
            let parent_commit = repo.head().unwrap().peel_to_commit().unwrap();
            let tree = repo.find_tree(oid).unwrap();
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Updated by Pithekos",
                &tree,
                &[&parent_commit],
            )
                .unwrap();
            status::Custom(
                Status::Ok,
                (
                    ContentType::JSON,
                    make_good_json_data_response("ok".to_string()),
                ),
            )
        }
        Err(e) => status::Custom(
            Status::InternalServerError,
            (
                ContentType::JSON,
                make_bad_json_data_response(
                    format!("could not add/commit repo: {}", e).to_string(),
                ),
            ),
        ),
    };
    result
}
