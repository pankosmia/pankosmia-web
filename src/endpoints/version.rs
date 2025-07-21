use rocket::get;
use rocket::http::ContentType;
use rocket::response::status;
use serde_json::json;
use crate::utils::response::ok_json_response;

/// *`GET /version`*
///
/// Typically mounted as **`/version`**
///
/// Returns an object containing version information
///
/// `{"pkg_version":"1.2.3"}`
#[get("/version")]
pub fn get_version() -> status::Custom<(ContentType, String)> {
    let crate_version = env!("CARGO_PKG_VERSION");
    let json_value = json!({"pkg_version": crate_version}).to_string();
    ok_json_response(json_value)
}