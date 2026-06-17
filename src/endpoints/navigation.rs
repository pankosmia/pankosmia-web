use crate::structs::{AppSettings, Bcv};
use crate::utils::files::write_app_state;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::response::{not_ok_json_response, ok_json_response, ok_ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, post, State};
use serde_json::json;

/// *`GET /bcv`*
///
/// Typically mounted as **`/navigation/bcv`**
///
/// Returns an object containing global BCV information
///
/// `{"book_code":"TIT","chapter":1,"verse":1, "to_verse": 1}`
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

/// *`POST /bcv/<book_code>/<chapter>/<verse>[/<to_verse>]`*
///
/// Typically mounted as **`/navigation/bcv/<book_code>/<chapter>/<verse>[/<to_verse>]`**
///
/// Sets global BCV with verse range.
#[post("/bcv/<book_code>/<chapter>/<verse>/<to_verse>")]
pub fn post_bcv_range(
    state: &State<AppSettings>,
    book_code: &str,
    chapter: u16,
    verse: u16,
    to_verse: u16,
) -> status::Custom<(ContentType, String)> {
    let new_bcv = Bcv {
        book_code: book_code.to_string(),
        chapter,
        verse: verse,
        to_verse: to_verse,
    };
    let new_state_json = json!(
        {
            "bcv": new_bcv.clone(),
            "current_project": state.current_project.lock().unwrap().clone(),
            "snippet": null,
            "word": null,
        }
    );
    match write_app_state(state, new_state_json) {
        Ok(_) => {}
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not write app state: '{}'", &e)),
            )
        }
    }
    *state.bcv.lock().unwrap() = new_bcv;
    ok_ok_json_response()
}

/// *`POST /bcv/<book_code>/<chapter>/<verse>`*
///
/// Typically mounted as **`/navigation/bcv/<book_code>/<chapter>/<verse>`**
///
/// Sets global BCV without verse range.
#[post("/bcv/<book_code>/<chapter>/<verse>")]
pub fn post_bcv(
    state: &State<AppSettings>,
    book_code: &str,
    chapter: u16,
    verse: u16,
) -> status::Custom<(ContentType, String)> {
    let new_bcv = Bcv {
        book_code: book_code.to_string(),
        chapter,
        verse: verse,
        to_verse: verse,
    };
    let new_state_json = json!(
        {
            "bcv": new_bcv.clone(),
            "current_project": state.current_project.lock().unwrap().clone(),
            "snippet": null,
            "word": null,
        }
    );
    match write_app_state(state, new_state_json) {
        Ok(_) => {}
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not write app state: '{}'", &e)),
            )
        }
    }
    *state.bcv.lock().unwrap() = new_bcv;
    let mut snippet_inner = state.snippet.lock().unwrap();
    *snippet_inner = None;
    let mut word_inner = state.word.lock().unwrap();
    *word_inner = None;
    ok_ok_json_response()
}
