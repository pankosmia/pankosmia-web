use std::sync::atomic::Ordering;
use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::{MsgQueue, NET_IS_ENABLED};
use crate::utils::json_responses::{make_good_json_data_response, make_net_status_response};

/// *`GET /status`*
///
/// Typically mounted as **`/net/status`**
///
/// Returns the current net-enable state as a JSON object.
///
/// `{"is_enabled":true}`
#[get("/status")]
pub fn net_status() -> status::Custom<(ContentType, String)> {
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            make_net_status_response(NET_IS_ENABLED.load(Ordering::Relaxed)),
        ),
    )
}

/// *`GET /enable`*
///
/// Typically mounted as **`/net/enable`**
///
/// Enables net state, returns a JSON OK response and generates an SSE notification.
///
/// `{"is_good":true,"reason":"ok"}`
#[get("/enable")]
pub fn net_enable(msgs: &State<MsgQueue>) -> status::Custom<(ContentType, String)> {
    msgs.lock()
        .unwrap()
        .push_back("info--5--net--enable".to_string());
    NET_IS_ENABLED.store(true, Ordering::Relaxed);
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
/// Typically mounted as **`/net/disable`**
///
/// Disables net state, returns a JSON OK response and generates an SSE notification.
///
/// `{"is_good":true,"reason":"ok"}`
#[get("/disable")]
pub fn net_disable(msgs: &State<MsgQueue>) -> status::Custom<(ContentType, String)> {
    msgs.lock()
        .unwrap()
        .push_back("info--5--net--disable".to_string());
    NET_IS_ENABLED.store(false, Ordering::Relaxed);
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            make_good_json_data_response("ok".to_string()),
        ),
    )
}