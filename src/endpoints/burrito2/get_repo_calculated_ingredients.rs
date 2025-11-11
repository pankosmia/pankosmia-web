use crate::structs::AppSettings;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{not_ok_bad_repo_json_response, ok_json_response};
use rocket::http::ContentType;
use rocket::response::status;
use rocket::{get, State};
use std::path::{Components, PathBuf};
use crate::utils::burrito::ingredients_metadata_from_files;

/// *`GET /calculated-ingredients/<repo_path>`*
///
/// Typically mounted as **`/burrito/calculated-ingredients/<repo_path>`**
///
/// Returns a map of files of ingredient information, calculated directly from the filesystem (not read from metadata.json).

#[get("/calculated-ingredients/<repo_path..>")]
pub async fn get_repo_calculated_ingredients(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let app_resources_dir = format!("{}", &state.app_resources_dir);
        let full_repo_dir = format!(
            "{}{}{}",
            state.repo_dir.lock().unwrap().clone(),
            os_slash_str(),
            &repo_path.display().to_string()
        );
        let ingredients = ingredients_metadata_from_files(app_resources_dir, full_repo_dir.clone());
        ok_json_response(serde_json::to_string(&ingredients).unwrap())
    } else {
        not_ok_bad_repo_json_response()
    }
}
