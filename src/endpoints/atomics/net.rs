use crate::static_vars::NET_IS_ENABLED;
use crate::utils::json_responses::{make_net_status_response};
use crate::utils::response::{ok_json_response, ok_ok_json_response};
use crate::MsgQueue;
use rocket::http::{ContentType};
use rocket::response::status;
use rocket::{get, post, State};
use std::sync::atomic::Ordering;

/// *`GET /status`*
///
/// Typically mounted as **`/net/status`**
///
/// Returns the current net-enable state as a JSON object.
///
/// `{"is_enabled":true}`
#[get("/status")]
pub fn net_status() -> status::Custom<(ContentType, String)> {
    ok_json_response(make_net_status_response(
        NET_IS_ENABLED.load(Ordering::Relaxed),
    ))
}

/// *`POST /enable`*
///
/// Typically mounted as **`/net/enable`**
///
/// Enables net state, returns a JSON OK response and generates an SSE notification.
///
/// `{"is_good":true,"reason":"ok"}`
#[post("/enable")]
pub fn net_enable(msgs: &State<MsgQueue>) -> status::Custom<(ContentType, String)> {
    msgs.lock()
        .unwrap()
        .push_back("info--5--net--enable".to_string());
    NET_IS_ENABLED.store(true, Ordering::Relaxed);
    ok_ok_json_response()
}

/// *`POST /disable`*
///
/// Typically mounted as **`/net/disable`**
///
/// Disables net state, returns a JSON OK response and generates an SSE notification.
///
/// `{"is_good":true,"reason":"ok"}`
#[post("/disable")]
pub fn net_disable(msgs: &State<MsgQueue>) -> status::Custom<(ContentType, String)> {
    msgs.lock()
        .unwrap()
        .push_back("info--5--net--disable".to_string());
    NET_IS_ENABLED.store(false, Ordering::Relaxed);
    ok_ok_json_response()
}
