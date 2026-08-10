use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::response::{not_ok_json_response, ok_ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{post, State};
use serde_json::Value;

/// *`POST /dev-setting/<setting_key>/<setting_value>`*
///
/// Typically mounted as **`/settings/dev-setting/<setting_key>/<setting_value>`**
///
/// Sets the value of a dev setting. setting_key must exist, setting_value must be a string
/// 
/// Prints new dev-server state, eg {"force_os": String("android")}
/// 
/// curl -X POST "http://localhost:19119/api/settings/dev-setting/force_os/android"
#[post("/dev-setting/<setting_key>/<setting_value>")]
pub fn post_dev_setting(
    state: &State<AppSettings>,
    setting_key: &str,
    setting_value: &str,
) -> status::Custom<(ContentType, String)> {
    let mut dev_settings_inner = state.dev_settings.lock().expect("lock dev settings");
    let mut new_value = dev_settings_inner.clone();
    let new_value_object: &mut serde_json::Map<std::string::String, serde_json::Value> =
        new_value.as_object_mut().expect("as object");
    if new_value_object.contains_key(setting_key) {
        new_value_object.insert(setting_key.to_string(), setting_value.into());
        println!("{:?}", &new_value_object);
        *dev_settings_inner = serde_json::to_value(new_value_object).expect("map to value");
    } else {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!("dev setting '{}' not found", &setting_key)),
        );
    };
    ok_ok_json_response()
}

/// *`POST /dev-setting/<setting_key>`*
///
/// Typically mounted as **`/settings/dev-setting/<setting_key>`**
///
/// Clears the value of a dev setting. setting_key must exist
/// 
/// Prints new dev-server state, eg {"force_os": Null}
/// 
/// curl -X POST "http://localhost:19119/api/settings/dev-setting/force_os"
#[post("/dev-setting/<setting_key>")]
pub fn post_clear_dev_setting(
    state: &State<AppSettings>,
    setting_key: &str,
) -> status::Custom<(ContentType, String)> {
    let mut dev_settings_inner = state.dev_settings.lock().expect("lock dev settings");
    let mut new_value = dev_settings_inner.clone();
    let new_value_object: &mut serde_json::Map<std::string::String, serde_json::Value> =
        new_value.as_object_mut().expect("as object");
    if new_value_object.contains_key(setting_key) {
        new_value_object.insert(setting_key.to_string(), Value::Null);
        println!("{:?}", &new_value_object);
        *dev_settings_inner = serde_json::to_value(new_value_object).expect("map to value");
    } else {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!("dev setting '{}' not found", &setting_key)),
        );
    };
    ok_ok_json_response()
}
