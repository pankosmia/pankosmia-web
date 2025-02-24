use std::path::PathBuf;
use regex::Regex;
use rocket::{get, post, State};
use rocket::response::{status, Redirect};
use rocket::http::{ContentType, Status};
use serde_json::json;

use crate::structs::{AppSettings, Typography, ContentOrRedirect};
use crate::utils::json_responses::{
    make_good_json_data_response,
    make_bad_json_data_response
};

#[get("/languages")]
pub(crate) fn get_languages(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let languages = state.languages.lock().unwrap().clone();
    match serde_json::to_string(&languages) {
        Ok(v) => status::Custom(Status::Ok, (ContentType::JSON, v)),
        Err(e) => status::Custom(
            Status::InternalServerError,
            (
                ContentType::JSON,
                make_bad_json_data_response(format!(
                    "Could not parse language settings as JSON array: {}",
                    e
                )),
            ),
        ),
    }
}

#[post("/languages/<languages..>")]
pub(crate) fn post_languages(
    state: &State<AppSettings>,
    languages: PathBuf
) -> status::Custom<(ContentType, String)> {
    let language_vec: Vec<String> = languages
        .display()
        .to_string()
        .split("/")
        .map(|s| s.to_string())
        .collect();
    if language_vec.len() == 0 {
        return status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response("No language code found".to_string()),
            ),
        )
    }
    let lang_regex = Regex::new(r"^[a-z]{2}$").unwrap();
    for lang in &language_vec {
        match lang_regex.find(&lang) {
            Some(_) => {},
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
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            make_good_json_data_response("ok".to_string()),
        ),
    )
}

#[get("/auth-token/<token_key>")]
pub(crate) fn get_auth_token(
    state: &State<AppSettings>,
    token_key: String,
) -> status::Custom<(ContentType, String)> {
    if !state.gitea_endpoints.contains_key(&token_key) {
        return status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response(format!(
                    "Unknown GITEA endpoint name: {}",
                    token_key
                )),
            ),
        );
    }
    let auth_tokens = state
        .auth_tokens
        .lock()
        .unwrap()
        .clone();
    if auth_tokens.contains_key(&token_key) {
        let code = &auth_tokens[&token_key];
        let ok_json = json!({"is_good": true, "code": code});
        match serde_json::to_string(&ok_json) {
            Ok(v) => status::Custom(
                Status::Ok,
                (
                    ContentType::JSON,
                    v
                ),
            ),
            Err(e) => status::Custom(
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!(
                        "Could not produce JSON for auth token: {}",
                        e
                    )),
                ),
            ),
        }
    } else {
        status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response(format!(
                    "Could not find record for token key '{}'",
                    token_key
                )),
            ),
        )
    }
}

#[get("/auth-token/<token_key>?<code>")]
pub(crate) fn get_new_auth_token(
    state: &State<AppSettings>,
    token_key: String,
    code: String,
) -> ContentOrRedirect {
    if !state.gitea_endpoints.contains_key(&token_key) {
        return ContentOrRedirect::Content(
            status::Custom(
                Status::BadRequest,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!(
                        "Unknown GITEA endpoint name: {}",
                        token_key
                    )),
                ),
            )
        );
    }
    let mut tokens_inner = state
        .auth_tokens
        .lock()
        .unwrap();
    if code == "" {
        tokens_inner.remove(&token_key);
    } else {
        tokens_inner.insert(token_key, code);
    }
    ContentOrRedirect::Redirect(Redirect::to("/clients/main"))
}

#[get("/typography")]
pub(crate) fn get_typography(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let typography = state.typography.lock().unwrap().clone();
    match serde_json::to_string(&typography) {
        Ok(v) => status::Custom(Status::Ok, (ContentType::JSON, v)),
        Err(e) => status::Custom(
            Status::InternalServerError,
            (
                ContentType::JSON,
                make_bad_json_data_response(format!(
                    "Could not parse typography settings as JSON object: {}",
                    e
                )),
            ),
        ),
    }
}

#[post("/typography/<font_set>/<size>/<direction>")]
pub(crate) fn post_typography(
    state: &State<AppSettings>,
    font_set: &str,
    size: &str,
    direction: &str,
) -> status::Custom<(ContentType, String)> {
    *state.typography.lock().unwrap() = Typography {
        font_set: font_set.to_string(),
        size: size.to_string(),
        direction: direction.to_string(),
    };
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            make_good_json_data_response("ok".to_string()),
        ),
    )
}