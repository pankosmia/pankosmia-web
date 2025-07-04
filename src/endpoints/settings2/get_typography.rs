use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::response::{not_ok_json_response, ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};

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
        Ok(v) => ok_json_response(v),
        Err(e) => not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!(
                "Could not parse typography settings as JSON object: {}",
                e
            )),
        ),
    }
}
