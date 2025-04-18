use std::collections::BTreeMap;
use std::path::PathBuf;
use regex::Regex;
use rocket::{get, post, State};
use rocket::response::{status, Redirect};
use rocket::http::{ContentType, Status, CookieJar};
use serde_json::json;
use crate::MsgQueue;
use crate::structs::{AppSettings, Typography, ContentOrRedirect, TypographyFeature};
use crate::utils::client::Clients;
use crate::utils::json_responses::{
    make_good_json_data_response,
    make_bad_json_data_response,
};
use crate::utils::paths::{os_slash_str, source_webfonts_path, webfonts_path};
use crate::utils::files::{copy_and_customize_webfont_css2, write_user_settings};

/// *`GET /languages`*
///
/// Typically mounted as **`/settings/languages`**
///
/// Returns an array containing the current selected UI languages.
///
/// `["en"]`
#[get("/languages")]
pub fn get_languages(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
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

/// *`GET /typography`*
///
/// Typically mounted as **`/settings/typography`**
///
/// Returns an array containing the current typography settings.
///
/// ```text
/// {
///   "font_set": "fonts-Pankosmia-CardoPankosmia-EzraSILPankosmia-PadaukPankosmia-AwamiNastaliqPankosmia-NotoNastaliqUrduPankosmia-GentiumPlus",
///   "size": "medium",
///   "direction": "ltr"
/// }
/// ```
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

/// *`POST /typography/<font_set>/<size>/<direction>`*
///
/// Typically mounted as **`/settings/typography/<font_set>/<size>/<direction>`**
///
/// Sets UI typography and interface direction
#[allow(irrefutable_let_patterns)]
#[post("/typography/<font_set>/<size>/<direction>")]
pub fn post_typography(
    state: &State<AppSettings>,
    clients: &State<Clients>,
    msgs: &State<MsgQueue>,
    font_set: &str,
    size: &str,
    direction: &str,
) -> status::Custom<(ContentType, String)> {
    if let mut typo_inner = state.typography.lock().unwrap() {
        let mut existing_features = BTreeMap::new();
        for (key, value) in &mut typo_inner.features {
            existing_features.insert(key.to_string(), value.to_vec());
        }
        *typo_inner = Typography {
            font_set: font_set.to_string(),
            size: size.to_string(),
            direction: direction.to_string(),
            features: existing_features,
        };
        msgs.lock()
            .unwrap()
            .push_back("info--3--typography--change".to_string());
    }
    match write_user_settings(&state, &clients) {
        Ok(_) => {}
        Err(e) => {
            return status::Custom(
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
    }
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            make_good_json_data_response("ok".to_string()),
        ),
    )
}

/// *`POST /typography-feature/<font_name>/<feature>/<value>`*
///
/// Typically mounted as **`/settings//typography-feature/<font_name>/<feature>/<value>`**
///
/// Sets the value of a font feature. Currently silently ignores unknown fonts and fields.
#[allow(irrefutable_let_patterns)]
#[post("/typography-feature/<font_name>/<feature>/<new_value>")]
pub fn post_typography_feature(
    state: &State<AppSettings>,
    clients: &State<Clients>,
    msgs: &State<MsgQueue>,
    font_name: &str,
    feature: &str,
    new_value: u8,
) -> status::Custom<(ContentType, String)> {
    if let mut typo_inner = state.typography.lock().unwrap() {
        let mut new_fonts = BTreeMap::new();
        for (font_key, font_value) in &mut typo_inner.features {
            if font_key == font_name {
                let mut new_fields = Vec::new();
                let font_inner = &mut *font_value;
                for field_kv in font_inner.clone() {
                    if field_kv.key == feature {
                        new_fields.push(TypographyFeature { key: field_kv.key.to_string(), value: new_value });
                    } else {
                        new_fields.push(TypographyFeature { key: field_kv.key.to_string(), value: field_kv.value });
                    }
                }
                let working_dir = state.working_dir.clone();
                let app_resources_dir = state.app_resources_dir.clone();
                let src_webfonts_dir = source_webfonts_path(&app_resources_dir);
                let target_webfonts_dir = webfonts_path(&working_dir);
                match copy_and_customize_webfont_css2(&src_webfonts_dir, &target_webfonts_dir, &new_fields, &font_name.to_string()) {
                    Ok(_) => {}
                    Err(e) => {
                        return status::Custom(
                            Status::InternalServerError,
                            (
                                ContentType::JSON,
                                make_bad_json_data_response(format!(
                                    "Could not rewrite CSS: {}",
                                    e
                                )),
                            ),
                        )
                    }
                }
                new_fonts.insert(font_key.to_string(), new_fields);
            } else {
                new_fonts.insert(font_key.to_string(), font_value.to_vec());
            }
        }
        *typo_inner = Typography {
            font_set: typo_inner.font_set.clone(),
            size: typo_inner.size.clone(),
            direction: typo_inner.direction.clone(),
            features: new_fonts,
        };
        msgs.lock()
            .unwrap()
            .push_back("info--3--typography-feature--change".to_string());
    }
    match write_user_settings(&state, &clients) {
        Ok(_) => {}
        Err(e) => {
            return status::Custom(
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
    }
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            make_good_json_data_response("ok".to_string()),
        ),
    )
}

// For testing only, remove this one day soon!
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

/// *`GET /auth-token/<token_key>/<code>/<client_code>`*
///
/// Typically mounted as **`/settings/auth-token/<token_key>/<code>/<client_code>`**
///
/// A landing URL for authentification via a gateway server.
///
/// `token_key` is the auth gateway label.
///
/// `code` is the code returned by that server, for future requests
///
/// `client_code` is a code generated by pankosmia-web to show that the incoming request corresponds to an earlier auth request.
///
/// A cookie called `<token_key>_code` is set on successful authentication.
#[get("/auth-token/<token_key>/<code>/<client_code>")]
pub fn get_new_auth_token<'a>(
    state: &State<AppSettings>,
    token_key: String,
    code: String,
    client_code: String,
    cj: &CookieJar<'_>,
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
    let mut auth_requests = state
        .auth_requests
        .lock()
        .unwrap();
    if !auth_requests.contains_key(&token_key) {
        return ContentOrRedirect::Content(
            status::Custom(
                Status::BadRequest,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!(
                        "No record auth request found for {}",
                        token_key
                    )),
                ),
            )
        );
    };
    let auth_request_record = auth_requests.get(&token_key).unwrap();
    if auth_request_record.code != client_code {
        return ContentOrRedirect::Content(
            status::Custom(
                Status::BadRequest,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!(
                        "Invalid client code for {}",
                        token_key
                    )),
                ),
            )
        );
    }
    let redirect_uri = format!("/{}", auth_request_record.redirect_uri.clone());
    auth_requests.remove(&token_key);
    let mut tokens_inner = state
        .auth_tokens
        .lock()
        .unwrap();
    if code == "" {
        cj.remove(format!("{}_code", token_key.clone()));
        tokens_inner.remove(&token_key);
    } else {
        tokens_inner.insert(token_key.clone(), code.clone());
        cj.add((format!("{}_code", token_key), code));
    }
    ContentOrRedirect::Redirect(Redirect::to(redirect_uri))
}
