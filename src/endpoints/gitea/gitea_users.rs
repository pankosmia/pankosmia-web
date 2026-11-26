use crate::static_vars::NET_IS_ENABLED;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::response::{
    not_ok_json_response, not_ok_offline_json_response, ok_json_response,
};
use rocket::get;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use serde_json::json;
use serde_json::Value;
use std::sync::atomic::Ordering;
use urlencoding::encode;

/// *`GET /users?<searchstring>`*
///
/// Typically mounted as **`/api/gitea/<gitea_server>?<search_string>`**
///
/// Returns an array of zero or more objects describing matched users.
///
/// ```text
/// [
///   {
///     "username": "johndoe",
///     "full_name": "John Z Doe"
///   }
/// ]
/// ```

#[get("/users/<gitea_server>?<search_string>")]
pub fn gitea_users(
    gitea_server: &str,
    search_string: &str,
) -> status::Custom<(ContentType, String)> {
    if !NET_IS_ENABLED.load(Ordering::Relaxed) {
        return not_ok_offline_json_response();
    }
    if search_string.len() < 2 {
        let empty_result = json!([]);
        return ok_json_response(serde_json::to_string(&empty_result).unwrap());
    }
    let encoded_query = encode(search_string);
    let gitea_path = format!(
        "https://{}/api/v1/users/search?q={}",
        gitea_server, encoded_query
    );
    match ureq::get(gitea_path.as_str()).call() {
        Ok(r) => match r.into_body().read_json::<Value>() {
            Ok(j) => {
                let response_data = j["data"].as_array().expect("response data array");
                let return_data: Vec<Value> = response_data
                    .iter()
                    .map(|r| -> Value {
                        json!({"name": r["username"], "full_name": r["full_name"]})
                    })
                    .collect();
                ok_json_response(serde_json::to_string(&return_data).unwrap())
            }
            Err(e) => not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!(
                    "could not serve GITEA server response as JSON string: {}",
                    e
                )),
            ),
        },
        Err(e) => not_ok_json_response(
            Status::BadGateway,
            make_bad_json_data_response(
                format!("could not read from GITEA server: {}", e).to_string(),
            ),
        ),
    }
}
