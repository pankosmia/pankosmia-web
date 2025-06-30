use std::path::{Components, PathBuf};
use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};

/// *`GET /metadata/raw/<repo_path>`*
///
/// Typically mounted as **`/burrito/metadata/raw/<repo_path>`**
///
/// Returns the raw metadata.json file for the specified burrito, where *repo_path* is *`<server>/<org>/<repo>`* and refers to a local repo.
#[get("/metadata/raw/<repo_path..>")]
pub async fn raw_metadata(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let path_to_serve = state.repo_dir.lock().unwrap().clone()
            + os_slash_str()
            + &repo_path.display().to_string()
            + "/metadata.json";
        match std::fs::read_to_string(path_to_serve) {
            Ok(v) => status::Custom(Status::Ok, (ContentType::JSON, v)),
            Err(e) => status::Custom(
                Status::BadRequest,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(
                        format!("could not read metadata: {}", e).to_string(),
                    ),
                ),
            ),
        }
    } else {
        status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response("bad repo path".to_string()),
            ),
        )
    }
}