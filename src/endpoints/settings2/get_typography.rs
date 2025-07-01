use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;

/// *`GET /typography`*
///
/// Typically mounted as **`/settings/typography`**
///
/// Returns an array containing the current typography settings.
///
/// ```text
/// {
///   "font_set": "fonts-Pankosmia-CardoPankosmia-EzraSILPankosmia-PadaukPankosmia-AwamiNastaliqPankosmia-NotoNastaliqUrduPankosmia-Gentium",
///   "size": "medium",
///   "direction": "ltr"
/// }
/// ```
#[get("/typography")]
pub(crate) fn get_typography(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let typography = state.typography.lock().unwrap().clone();
    match serde_json::to_string(&typography) {
        Ok(v) => status::Custom(Status::Ok, (ContentType::JSON, v)),
        Err(e) => status::Custom(
            Status::InternalServerError,
            (
                ContentType::JSON,
                make_bad_json_data_response(format!(
                    "Could not parse typography settings as JSON object: {}",
                    e
                )),
            ),
        ),
    }
}