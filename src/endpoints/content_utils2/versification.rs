use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;

/// *`GET /versification/<versification_name>`*
///
/// Typically mounted as **`/content-utils/versification/<versification_name>`**
///
/// Returns chapter/verse info for a given versification as JSON
#[get("/versification/<versification_name>")]
pub async fn versification(
    state: &State<AppSettings>,
    versification_name: String,
) -> status::Custom<(ContentType, String)> {
    let path_to_serve = format!(
        "{}{}{}{}{}{}{}{}{}",
        &state.app_resources_dir,
        os_slash_str(),
        "templates",
        os_slash_str(),
        "content_templates",
        os_slash_str(),
        "vrs",
        os_slash_str(),
        versification_name.clone() + ".json"
    );

    match std::fs::read_to_string(path_to_serve) {
        Ok(v) => {
            status::Custom(
                Status::Ok,
                (
                    ContentType::JSON,
                    v,
                ),
            )
        }
        Err(e) => status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response(
                    format!("could not read versification file for '{}': {}", versification_name, e).to_string(),
                ),
            ),
        ),
    }
}
