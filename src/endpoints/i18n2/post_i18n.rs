use crate::static_vars::I18N_UPDATE_COUNT;
use crate::structs::AppSettings;
use crate::utils::json_responses::{make_bad_json_data_response};
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, State};
use serde_json::Value;
use std::sync::atomic::Ordering;

/// *`POST /`*
///
/// Typically mounted as **`/i18n`**
///
/// Replaces the local i18n file.
#[post("/", format = "json", data = "<payload>")]
pub async fn post_i18n(
    state: &State<AppSettings>,
    payload: Json<Value>,
) -> status::Custom<(ContentType, String)> {
    let serialized = payload.to_string();
    let save_path = state.working_dir.clone() + os_slash_str() + "i18n.json";
    match std::fs::write(save_path, serialized) {
        Ok(_) => {
            let current_i18n_count = I18N_UPDATE_COUNT.load(Ordering::Relaxed);
            I18N_UPDATE_COUNT.store(current_i18n_count + 1, Ordering::Relaxed);
            ok_ok_json_response()
        }
        Err(e) => not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response(format!("could not write i18n: {}", e).to_string()),
        ),
    }
}
