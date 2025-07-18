use crate::structs::AppSettings;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{not_ok_bad_repo_json_response, ok_json_response};
use rocket::http::ContentType;
use rocket::response::status;
use rocket::{get, State};
use std::path::{Components, Path, PathBuf};
use walkdir::WalkDir;

/// *`GET /paths/<repo_path>`*
///
/// Typically mounted as **`/burrito/paths/<repo_path>`**
///
/// Returns an array of files (not indexed ingredients) in a repo. Hidden files and directories are ignored.

#[get("/paths/<repo_path..>")]
pub async fn get_repo_file_paths(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let full_repo_dir = format!(
            "{}{}{}/ingredients/",
            state.repo_dir.lock().unwrap().clone(),
            os_slash_str(),
            &repo_path.display().to_string()
        );
        let mut paths = Vec::new();
        for entry in WalkDir::new(&full_repo_dir) {
            let entry_string = entry.unwrap().path().display().to_string();
            if Path::new(&entry_string).is_file() {
                let truncated_entry_string = entry_string.replace(&full_repo_dir, "");
                if !truncated_entry_string.starts_with(".") && !truncated_entry_string.contains(format!("{}.", os_slash_str()).as_str()) {
                    paths.push(truncated_entry_string);
                }
            }
        }
        ok_json_response(serde_json::to_string(&paths).unwrap())
    } else {
        not_ok_bad_repo_json_response()
    }
}
