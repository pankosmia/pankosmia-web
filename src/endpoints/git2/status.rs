use crate::structs::{AppSettings, GitStatusRecord};
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::response::{not_ok_json_response, ok_json_response};
use git2::{Repository, StatusOptions};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};
use std::path::PathBuf;
use crate::utils::git::git_repo_path;

/// *`GET /status/<repo_path>`*
///
/// Typically mounted as **`/git/status/<repo_path>`**
///
/// Returns an array of changes to the local repo from the given repo path.
///
/// ```text
/// [
///   {
///     "path": "ingredients/LICENSE.md",
///     "change_type": "modified"
///   }
/// ]
/// ```
#[get("/status/<repo_path..>")]
pub async fn git_status(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let repo_dir_string = state.repo_dir.lock().unwrap().clone();
    let repo_url_string = repo_path.display().to_string();
    let repo_path_string = match git_repo_path(
        repo_dir_string.clone(),
        repo_url_string.clone()
    ) {
        Some(s) => s,
        None => {
            return not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(format!("repo directory '{}' does not exist or does not contain a .git directory", &repo_url_string)),
                );
            }
    };
    match Repository::open(repo_path_string) {
        Ok(repo) => {
            if repo.is_bare() {
                return not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response("cannot get status of bare repo".to_string()),
                );
            };
            let mut opts = StatusOptions::new();
            opts.include_untracked(true);
            match repo.statuses(Some(&mut opts)) {
                Ok(statuses) => {
                    let mut status_changes = Vec::new();
                    for entry in statuses
                        .iter()
                        .filter(|e| e.status() != git2::Status::CURRENT)
                    {
                        let i_status = match entry.status() {
                            s if s.contains(git2::Status::INDEX_NEW)
                                || s.contains(git2::Status::WT_NEW) =>
                            {
                                "new"
                            }
                            s if s.contains(git2::Status::INDEX_MODIFIED)
                                || s.contains(git2::Status::WT_MODIFIED) =>
                            {
                                "modified"
                            }
                            s if s.contains(git2::Status::INDEX_DELETED)
                                || s.contains(git2::Status::WT_DELETED) =>
                            {
                                "deleted"
                            }
                            s if s.contains(git2::Status::INDEX_RENAMED)
                                || s.contains(git2::Status::WT_RENAMED) =>
                            {
                                "renamed"
                            }
                            s if s.contains(git2::Status::INDEX_TYPECHANGE)
                                || s.contains(git2::Status::WT_TYPECHANGE) =>
                            {
                                "type_change"
                            }
                            _ => "",
                        };

                        if entry.status().contains(git2::Status::IGNORED) {
                            continue;
                        }
                        status_changes.push(GitStatusRecord {
                            path: entry.path().unwrap().to_string(),
                            change_type: i_status.to_string(),
                        });
                    }
                    ok_json_response(serde_json::to_string_pretty(&status_changes).unwrap())
                }
                Err(e) => not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(
                        format!("could not get repo status: {}", e).to_string(),
                    ),
                ),
            }
        }
        Err(e) => not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response(format!("could not open repo: {}", e).to_string()),
        ),
    }
}
