use crate::structs::{AppSettings, Bcv};
use crate::utils::json_responses::{make_bad_json_data_response};
use crate::utils::response::{not_ok_json_response, ok_json_response, ok_ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, post, State};

/// *`GET /bcv`*
///
/// Typically mounted as **`/navigation/bcv`**
///
/// Returns an object containing global BCV information
///
/// `{"book_code":"TIT","chapter":1,"verse":1}`
#[get("/bcv")]
pub fn get_bcv(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let bcv = state.bcv.lock().unwrap().clone();
    match serde_json::to_string(&bcv) {
        Ok(v) => ok_json_response(v),
        Err(e) => not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response(format!("Could not parse bcv state as JSON object: {}", e)),
        ),
    }
}

/// *`POST /bcv/<book_code>/<chapter>/<verse>`*
///
/// Typically mounted as **`/navigation/bcv/<book_code>/<chapter>/<verse>`**
///
/// Sets global BCV.
#[post("/bcv/<book_code>/<chapter>/<verse>")]
pub fn post_bcv(
    state: &State<AppSettings>,
    book_code: &str,
    chapter: u16,
    verse: u16,
) -> status::Custom<(ContentType, String)> {
    *state.bcv.lock().unwrap() = Bcv {
        book_code: book_code.to_string(),
        chapter,
        verse,
    };
    ok_ok_json_response()
}
