use crate::structs::AppSettings;
use crate::utils::burrito_api::audit::check_basic_shape;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, ok_json_response,
};
use rocket::http::ContentType;
use rocket::response::status;
use rocket::{get, State};
use std::path::{Components, PathBuf};

/// *`GET /audit/<repo_path>`*
///
/// Typically mounted as **`/burrito/audit/<repo_path>`**
///
/// Returns a report about the specified burrito, where *repo_path* is *`<server>/<org>/<repo>`* and refers to a local repo.
#[get("/audit/<repo_path..>")]
pub async fn audit(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let burrito_path = state.repo_dir.lock().unwrap().clone()
            + os_slash_str()
            + &repo_path.display().to_string();
        let report = check_basic_shape(burrito_path);
        ok_json_response(serde_json::to_string(&report).unwrap())
    } else {
        not_ok_bad_repo_json_response()
    }
}
