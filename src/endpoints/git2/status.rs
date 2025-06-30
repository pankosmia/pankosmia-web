use std::path::PathBuf;
use git2::{Repository, StatusOptions};
use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::structs::{AppSettings, GitStatusRecord};
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;

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
    let repo_path_string: String = state.repo_dir.lock().unwrap().clone()
        + os_slash_str()
        + &repo_path.display().to_string().clone();
    match Repository::open(repo_path_string) {
        Ok(repo) => {
            if repo.is_bare() {
                return status::Custom(
                    Status::BadRequest,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response("cannot get status of bare repo".to_string()),
                    ),
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
                        // println!("{} ({})", entry.path().unwrap(), istatus);
                    }
                    status::Custom(
                        Status::Ok,
                        (
                            ContentType::JSON,
                            serde_json::to_string_pretty(&status_changes).unwrap(),
                        ),
                    )
                }
                Err(e) => status::Custom(
                    Status::InternalServerError,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response(
                            format!("could not get repo status: {}", e).to_string(),
                        ),
                    ),
                ),
            }
        }
        Err(e) => status::Custom(
            Status::InternalServerError,
            (
                ContentType::JSON,
                make_bad_json_data_response(format!("could not open repo: {}", e).to_string()),
            ),
        ),
    }
}
