use std::path::PathBuf;
use regex::Regex;
use rocket::{post, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::structs::AppSettings;
use crate::utils::client::Clients;
use crate::utils::files::write_user_settings;
use crate::utils::json_responses::{make_bad_json_data_response, make_good_json_data_response};
use crate::utils::paths::os_slash_str;

/// *`POST /languages/<lang>/<lang>/...`*
///
/// Typically mounted as **`/languages/<lang>/<lang>/...`**
///
/// Sets UI languages
#[post("/languages/<languages..>")]
pub fn post_languages(
    state: &State<AppSettings>,
    clients: &State<Clients>,
    languages: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let language_vec: Vec<String> = languages
        .display()
        .to_string()
        .split(os_slash_str())
        .map(|s| s.to_string())
        .collect();
    if language_vec.len() == 0 {
        return status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response("No language code found".to_string()),
            ),
        );
    }
    let lang_regex = Regex::new(r"^[a-z]{2,3}$").unwrap();
    for lang in &language_vec {
        match lang_regex.find(&lang) {
            Some(_) => {}
            None => return status::Custom(
                Status::BadRequest,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!(
                        "Bad language code: {}",
                        lang
                    )),
                ),
            )
        }
    }
    *state.languages.lock().unwrap() = language_vec;
    match write_user_settings(&state, &clients) {
        Ok(_) => {}
        Err(e) => return status::Custom(
            Status::InternalServerError,
            (
                ContentType::JSON,
                make_bad_json_data_response(format!(
                    "Could not write out user settings: {}",
                    e
                )),
            ),
        )
    }
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            make_good_json_data_response("ok".to_string()),
        ),
    )
}