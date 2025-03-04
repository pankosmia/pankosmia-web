use rocket::{get, post, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::structs::{AppSettings, Bcv};
use crate::utils::json_responses::{make_bad_json_data_response, make_good_json_data_response};

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
        Ok(v) => status::Custom(Status::Ok, (ContentType::JSON, v)),
        Err(e) => status::Custom(
            Status::InternalServerError,
            (
                ContentType::JSON,
                make_bad_json_data_response(format!(
                    "Could not parse bcv state as JSON object: {}",
                    e
                )),
            ),
        ),
    }
}

/// *`POST /bcv/<book_code>/<chapter>/<verse>`*
///
/// Typically mounted as **`/navigation/bcv/<book_code>/<chapter>/<verse>`**
///
/// Sets global BCV
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
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            make_good_json_data_response("ok".to_string()),
        ),
    )
}

