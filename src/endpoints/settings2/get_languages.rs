use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;

/// *`GET /languages`*
///
/// Typically mounted as **`/settings/languages`**
///
/// Returns an array containing the current selected UI languages.
///
/// `["en"]`
#[get("/languages")]
pub fn get_languages(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let languages = state.languages.lock().unwrap().clone();
    match serde_json::to_string(&languages) {
        Ok(v) => status::Custom(Status::Ok, (ContentType::JSON, v)),
        Err(e) => status::Custom(
            Status::InternalServerError,
            (
                ContentType::JSON,
                make_bad_json_data_response(format!(
                    "Could not parse language settings as JSON array: {}",
                    e
                )),
            ),
        ),
    }
}