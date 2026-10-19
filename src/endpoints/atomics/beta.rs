use crate::static_vars::BETA_IS_ENABLED;
use crate::utils::json_responses::{make_net_status_response};
use crate::utils::response::{ok_ok_json_response, ok_json_response};
use crate::MsgQueue;
use rocket::http::{ContentType};
use rocket::response::status;
use rocket::{get, State};
use std::sync::atomic::Ordering;

/// *`GET /status`*
///
/// Typically mounted as **`/beta/status`**
///
/// Returns the current beta state as a JSON object.
///
/// `{"is_enabled":true}`
#[get("/status")]
pub fn beta_status() -> status::Custom<(ContentType, String)> {
    ok_json_response(make_net_status_response(
        BETA_IS_ENABLED.load(Ordering::Relaxed),
    ))
}

/// *`GET /enable`*
///
/// Typically mounted as **`/beta/enable`**
///
/// Enables beta state, returns a JSON OK response and generates an SSE notification.
///
/// `{"is_good":true,"reason":"ok"}`
#[get("/enable")]
pub fn beta_enable(msgs: &State<MsgQueue>) -> status::Custom<(ContentType, String)> {
    msgs.lock()
        .unwrap()
        .push_back("info--5--beta--enable".to_string());
    BETA_IS_ENABLED.store(true, Ordering::Relaxed);
    ok_ok_json_response()
}

/// *`GET /disable`*
///
/// Typically mounted as **`/beta/disable`**
///
/// Disables beta state, returns a JSON OK response and generates an SSE notification.
///
/// `{"is_good":true,"reason":"ok"}`
#[get("/disable")]
pub fn beta_disable(msgs: &State<MsgQueue>) -> status::Custom<(ContentType, String)> {
    msgs.lock()
        .unwrap()
        .push_back("info--5--beta--disable".to_string());
    BETA_IS_ENABLED.store(false, Ordering::Relaxed);
    ok_ok_json_response()
}
