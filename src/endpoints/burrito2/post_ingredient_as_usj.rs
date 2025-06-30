use std::path::{Components, PathBuf};
use hallomai::transform;
use rocket::{post, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use serde_json::Value;
use crate::structs::AppSettings;
use crate::utils::json_responses::{make_bad_json_data_response, make_good_json_data_response};
use crate::utils::paths::{check_path_components, check_path_string_components, os_slash_str};

/// *`POST /ingredient/as-usj/<repo_path>?ipath=my_burrito_path`*
///
/// Typically mounted as **`/burrito/ingredient/as-usj/<repo_path>?ipath=my_burrito_path`**
///
/// Writes a USJ documents as a USFM ingredient, where the document is provided as an HTTP form file.
/// The USFM file must exist, ie this is not the way to add new ingredients to a Burrito
/// Currently slow and buggy but works for typical CCBT USFM.
/// Adding a hack to avoid usfm tag weirdness

#[post(
    "/ingredient/as-usj/<repo_path..>?<ipath>",
    format = "json",
    data = "<json_form>"
)]
pub async fn post_ingredient_as_usj(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    ipath: String,
    json_form: Json<Value>,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    let destination = state.repo_dir.lock().unwrap().clone()
        + os_slash_str()
        + &repo_path.display().to_string()
        + "/ingredients/"
        + ipath.clone().as_str();
    if check_path_components(&mut path_components.clone())
        && check_path_string_components(ipath.clone())
        && std::fs::metadata(destination.clone()).is_ok()
    {
        let usfm = transform(json_form.to_string(), "usj".to_string(), "usfm".to_string())
            .replace("\\usfm 0.2.1\n", "");
        match std::fs::write(destination, usfm) {
            Ok(_) => status::Custom(
                Status::Ok,
                (
                    ContentType::JSON,
                    make_good_json_data_response("ok".to_string()),
                ),
            ),
            Err(e) => status::Custom(
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!("Could not write to {}: {}", ipath, e)),
                ),
            )
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