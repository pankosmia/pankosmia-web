use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_json_response,
};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};
use std::path::{Components, Path, PathBuf};
use walkdir::WalkDir;
use serde_json::json;

/// *`GET /paths-info/<repo_path>`*
///
/// Typically mounted as **`/burrito/paths-info/<repo_path>`**
///
/// Returns an array of objects containing path, size and update time for ingredient paths (not indexed ingredients) in a repo. Hidden files and directories are ignored.

#[get("/paths-info/<repo_path..>")]
pub async fn get_repo_file_paths_info(
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
        if !Path::new(&full_repo_dir).exists() {
            return ok_json_response("[]".to_string());
        }
        let mut paths = Vec::new();
        for entry in WalkDir::new(&full_repo_dir) {
            let entry_string = match entry {
                Ok(ent) => ent.path().display().to_string(),
                Err(e) => {
                    return not_ok_json_response(
                        Status::BadRequest,
                        make_bad_json_data_response(
                            format!("could not read entry: {}", e).to_string(),
                        ),
                    );
                }
            };
            if Path::new(&entry_string).is_file() {
                let truncated_entry_string = entry_string.replace(&full_repo_dir, "");
                if !truncated_entry_string.starts_with(".")
                    && !truncated_entry_string.ends_with(".bak")
                    && !truncated_entry_string.contains(format!("{}.", os_slash_str()).as_str())
                {
                    let path_path = truncated_entry_string.replace("\\", "/");
                    let path_metadata = std::fs::metadata(&entry_string).expect("path metadata");
                    let path_length = path_metadata.len();
                    let path_time = path_metadata.modified().expect("path time").duration_since(std::time::SystemTime::UNIX_EPOCH).expect("path duration").as_secs();
                    paths.push(json!({"path": path_path, "size": path_length, "modified_epoch": path_time}));
                }
            }
        }
        ok_json_response(serde_json::to_string(&paths).unwrap())
    } else {
        not_ok_bad_repo_json_response()
    }
}
