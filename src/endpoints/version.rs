use rocket::{get, State};
use rocket::http::ContentType;
use rocket::response::status;
use serde_json::json;
use crate::structs::AppSettings;
use crate::utils::response::ok_json_response;

/// *`GET /version`*
///
/// Typically mounted as **`/version`**
///
/// Returns an object containing version information
///
/// `{"pkg_version":"1.2.3"}`
#[get("/version")]
pub fn get_version(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let crate_version = env!("CARGO_PKG_VERSION");
    let product = &state.product;
    let json_value = json!({
        "pkg_version": crate_version,
        "product_name": product.name,
        "product_short_name": product.short_name,
        "product_version": product.version,
        "product_date_time": product.date_time,
    })
        .to_string();
    ok_json_response(json_value)
}