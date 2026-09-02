use rocket::{Shutdown, post};
use rocket::response::status;
use rocket::http::ContentType;
use crate::utils::response::ok_ok_json_response;

/// *`GET /shutdown`*
///
/// Typically mounted as **`/api/system/shutdown`**
///
/// Performs a graceful shutdown of the server.

#[post("/shutdown")]
pub fn shutdown(shutdown: Shutdown) -> status::Custom<(ContentType, String)> {
    shutdown.notify();
    ok_ok_json_response()
}