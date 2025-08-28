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
use rocket::serde::Deserialize;
use rocket::serde::json::Json;

/// *`POST /add-and-commit/<repo_path>`*
///
/// Typically mounted as **`/git/add-and-commit/<repo_path>`**
///
/// Adds and commits modified files for a given repo.

#[derive(Deserialize)]
pub struct AddCommitForm {
    commit_message: String,
}

#[post("/add-and-commit/<repo_path..>", format = "json", data = "<json_form>")]
pub async fn add_and_commit(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    json_form: Json<AddCommitForm>,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let repo_path_string = format!(
            "{}{}{}",
            state.repo_dir.lock().unwrap().clone(),
            os_slash_str(),
            &repo_path.display().to_string().clone()
        );
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
                    json_form.commit_message.as_str(),
                    &tree,
                    &[&parent_commit],
                )
                .unwrap();
                ok_ok_json_response()
            }
            Err(e) => not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(
                    format!("could not open repo: {}", e).to_string(),
                ),
            ),
        };
        result
    } else {
        not_ok_bad_repo_json_response()
    }
}
