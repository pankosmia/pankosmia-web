use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, check_path_string_components, os_slash_str};
use crate::utils::response::{not_ok_json_response, ok_json_response, not_ok_bad_repo_json_response};
use hallomai::transform;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};
use std::path::{Components, PathBuf};

/// *`GET /ingredient/as-usj/<repo_path>?ipath=my_burrito_path`*
///
/// Typically mounted as **`/burrito/ingredient/as-usj/<repo_path>?ipath=my_burrito_path`**
///
/// Returns a USFM resource as USJ. Currently slow and buggy but works for typical CCBT USFM.
#[get("/ingredient/as-usj/<repo_path..>?<ipath>")]
pub async fn get_ingredient_as_usj(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    ipath: String,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone())
        && check_path_string_components(ipath.clone())
    {
        let path_to_serve = state.repo_dir.lock().unwrap().clone()
            + os_slash_str()
            + &repo_path.display().to_string()
            + "/ingredients/"
            + ipath.as_str();
        match std::fs::read_to_string(path_to_serve) {
            Ok(v) => ok_json_response(transform(v, "usfm".to_string(), "usj".to_string())),
            Err(e) => not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(
                    format!("could not read ingredient content: {}", e).to_string(),
                ),
            ),
        }
    } else {
        not_ok_bad_repo_json_response()
    }
}
