use std::sync::atomic::Ordering;
use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::MsgQueue;
use crate::static_vars::DEBUG_IS_ENABLED;
use crate::utils::json_responses::{make_good_json_data_response, make_net_status_response};

/// *`GET /status`*
///
/// Typically mounted as **`/debug/status`**
///
/// Returns the current debug state as a JSON object.
///
/// `{"is_enabled":true}`
#[get("/status")]
pub fn debug_status() -> status::Custom<(ContentType, String)> {
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            make_net_status_response(DEBUG_IS_ENABLED.load(Ordering::Relaxed)),
        ),
    )
}

/// *`GET /enable`*
///
/// Typically mounted as **`/debug/enable`**
///
/// Enables debug state, returns a JSON OK response and generates an SSE notification.
///
/// `{"is_good":true,"reason":"ok"}`
#[get("/enable")]
pub fn debug_enable(msgs: &State<MsgQueue>) -> status::Custom<(ContentType, String)> {
    msgs.lock()
        .unwrap()
        .push_back("info--5--debug--enable".to_string());
    DEBUG_IS_ENABLED.store(true, Ordering::Relaxed);
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            make_good_json_data_response("ok".to_string()),
        ),
    )
}

/// *`GET /disable`*
///
/// Typically mounted as **`/debug/disable`**
///
/// Disables debug state, returns a JSON OK response and generates an SSE notification.
///
/// `{"is_good":true,"reason":"ok"}`
#[get("/disable")]
pub fn debug_disable(msgs: &State<MsgQueue>) -> status::Custom<(ContentType, String)> {
    msgs.lock()
        .unwrap()
        .push_back("info--5--debug--disable".to_string());
    DEBUG_IS_ENABLED.store(false, Ordering::Relaxed);
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            make_good_json_data_response("ok".to_string()),
        ),
    )
}