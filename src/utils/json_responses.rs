#[path="../structs.rs"]
mod structs;
use structs::{JsonDataResponse, JsonNetStatusResponse};

pub(crate) fn make_json_data_response(is_good: bool, reason: String) -> String {
    let jr: JsonDataResponse = JsonDataResponse { is_good, reason };
    serde_json::to_string(&jr).unwrap()
}

pub(crate) fn make_net_status_response(is_enabled: bool) -> String {
    let nsr: JsonNetStatusResponse = JsonNetStatusResponse { is_enabled };
    serde_json::to_string(&nsr).unwrap()
}
pub(crate) fn make_bad_json_data_response(reason: String) -> String {
    make_json_data_response(false, reason)
}

pub(crate) fn make_good_json_data_response(reason: String) -> String {
    make_json_data_response(true, reason)
}