use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::response::not_ok_json_response;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};

/// *`GET /log`*
///
/// Typically mounted as **`/system/log`**
///
/// Returns the raw log file if one is configured, otherwise 404.
#[get("/log")]
pub async fn raw_log(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    match state.log_path.clone() {
        Some(lp) => match std::fs::read_to_string(&lp) {
                Ok(lc) => return status::Custom(
                Status::Ok,
                (ContentType::new("application", "text/plain"), lc),
            ),
            Err(e) => not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response(format!("Could not read log file at {}: {}", &lp, e)),
        )
            },
        None => not_ok_json_response(
            Status::NotFound,
            make_bad_json_data_response("No log file configured".to_string()),
        ),
    }
}
