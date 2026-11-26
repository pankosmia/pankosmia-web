use crate::structs::{JsonDataResponse, JsonAugmentedDataResponse, JsonNetStatusResponse};

pub(crate) fn make_json_data_response(is_good: bool, reason: String) -> String {
    let jr: JsonDataResponse = JsonDataResponse { is_good, reason };
    serde_json::to_string(&jr).unwrap()
}

pub(crate) fn make_augmented_json_data_response(is_good: bool, reason_tuple: (bool, String, String)) -> String {
    let jr: JsonAugmentedDataResponse = JsonAugmentedDataResponse { is_good, reason_key: reason_tuple.1, reason: reason_tuple.2 };
    serde_json::to_string(&jr).unwrap()
}

pub(crate) fn make_net_status_response(is_enabled: bool) -> String {
    let nsr: JsonNetStatusResponse = JsonNetStatusResponse { is_enabled };
    serde_json::to_string(&nsr).unwrap()
}
pub(crate) fn make_bad_json_data_response(reason: String) -> String {
    make_json_data_response(false, reason)
}

pub(crate) fn make_augmented_bad_json_data_response(reason_tuple: (bool, String, String)) -> String {
    let jr: JsonAugmentedDataResponse = JsonAugmentedDataResponse { is_good: false, reason_key: reason_tuple.1, reason: reason_tuple.2 };
    serde_json::to_string(&jr).unwrap()
}

pub(crate) fn make_good_json_data_response(reason: String) -> String {
    make_json_data_response(true, reason)
}

pub(crate) fn make_augmented_good_json_data_response(reason_tuple: (bool, String, String)) -> String {
    let jr: JsonAugmentedDataResponse = JsonAugmentedDataResponse { is_good: true, reason_key: reason_tuple.1, reason: reason_tuple.2 };
    serde_json::to_string(&jr).unwrap()
}
