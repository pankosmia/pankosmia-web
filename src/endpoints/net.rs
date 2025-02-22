use std::sync::atomic::Ordering;
use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::{MsgQueue, NET_IS_ENABLED};
use crate::utils::json_responses::{make_good_json_data_response, make_net_status_response};

#[get("/status")]
pub(crate) fn net_status() -> status::Custom<(ContentType, String)> {
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            make_net_status_response(NET_IS_ENABLED.load(Ordering::Relaxed)),
        ),
    )
}

#[get("/enable")]
pub(crate) fn net_enable(msgs: &State<MsgQueue>) -> status::Custom<(ContentType, String)> {
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

#[get("/disable")]
pub(crate) fn net_disable(msgs: &State<MsgQueue>) -> status::Custom<(ContentType, String)> {
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