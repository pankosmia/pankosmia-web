use crate::structs::AppSettings;
use crate::structs::BytesOrError;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};
use std::path::{Components, PathBuf};
use crate::utils::zip::{make_zip_file};

/// *`GET /zipped/<repo_path>`*
///
/// Typically mounted as **`/burrito/zipped/<repo_path>`**
///
/// Returns a zip of a repo.
#[get("/zipped/<repo_path..>")]
pub async fn get_zipped_repo(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, BytesOrError)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone())
    {
        // Find directory
        let path_to_repo = format!(
            "{}{}{}",
            state.repo_dir.lock().unwrap().clone(),
            os_slash_str(),
            &repo_path.display().to_string()
        );
        if !std::path::Path::new(&path_to_repo).is_dir() {
            return status::Custom(
                Status::BadRequest,
                (
                    ContentType::JSON,
                    BytesOrError::Error(
                        make_bad_json_data_response(
                            format!("could not locate repo").to_string(),
                        ),
                    ),
                ),
            );
        }
        let temp_zip_path = make_zip_file(&path_to_repo);
        match std::fs::read(&temp_zip_path) {
            Ok(b) => status::Custom(
                Status::Ok,
                (
                    ContentType::ZIP,
                    BytesOrError::Bytes(b),
                ),
            ),
            Err(e) => status::Custom(
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    BytesOrError::Error(make_bad_json_data_response(format!("Could not read zip: {}", e))),
                ),
            )
        }
    } else {
        status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                BytesOrError::Error(make_bad_json_data_response("bad repo path".to_string())),
            ),
        )
    }
}
