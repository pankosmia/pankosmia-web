use crate::static_vars::NET_IS_ENABLED;
use crate::structs::{AppSettings, ContentOrRedirect};
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::response::{not_ok_json_response, not_ok_offline_json_response};
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket::{get, State};
use std::sync::atomic::Ordering;
/// *`GET /logout/<auth_key>`* (uses cookie with key `<auth_key>_code`)
///
/// Typically mounted as **`/gitea/logout/<auth_key>`**
///
/// Logs out of a remote server.
#[get("/logout/<token_key>")]
pub fn gitea_proxy_logout(
    state: &State<AppSettings>,
    token_key: String,
    cj: &CookieJar<'_>,
) -> ContentOrRedirect {
    if !NET_IS_ENABLED.load(Ordering::Relaxed) {
        return ContentOrRedirect::Content(not_ok_offline_json_response());
    }
    if !state.gitea_endpoints.contains_key(&token_key) {
        return ContentOrRedirect::Content(not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!("Unknown GITEA endpoint name: {}", token_key)),
        ));
    }
    // Get the cookie;
    let cookie_name = format!("{}_code", token_key);
    let cookie_code = match cj.get(cookie_name.as_str()) {
        Some(c) => c.value(),
        None => "",
    };
    // Logout of proxy server
    let logout_url = format!(
        "{}/logout?client_code={}",
        state.gitea_endpoints[&token_key].clone(),
        cookie_code
    );
    match ureq::get(logout_url.as_str()).call() {
        Ok(_) => {
            // Remove any existing token
            state.auth_tokens.lock().unwrap().remove(&token_key);
            // Remove cookie
            cj.remove(cookie_name);
            // Do redirect
            ContentOrRedirect::Redirect(Redirect::to("/"))
        }
        Err(e) => ContentOrRedirect::Content(not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response(format!("Error on logout from proxy {}: {}", token_key, e)),
        )),
    }
}
