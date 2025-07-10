use crate::structs::{AppSettings, Typography};
use crate::utils::client::Clients;
use crate::utils::files::write_user_settings;
use crate::utils::json_responses::{make_bad_json_data_response};
use crate::utils::response::{not_ok_json_response, ok_ok_json_response};
use crate::MsgQueue;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{post, State};
use std::collections::BTreeMap;

/// *`POST /typography/<font_set>/<size>/<direction>`*
///
/// Typically mounted as **`/settings/typography/<font_set>/<size>/<direction>`**
///
/// Sets UI typography and interface direction
#[allow(irrefutable_let_patterns)]
#[post("/typography/<font_set>/<size>/<direction>")]
pub fn post_typography(
    state: &State<AppSettings>,
    clients: &State<Clients>,
    msgs: &State<MsgQueue>,
    font_set: &str,
    size: &str,
    direction: &str,
) -> status::Custom<(ContentType, String)> {
    if let mut typo_inner = state.typography.lock().unwrap() {
        let mut existing_features = BTreeMap::new();
        for (key, value) in &mut typo_inner.features {
            existing_features.insert(key.to_string(), value.to_vec());
        }
        *typo_inner = Typography {
            font_set: font_set.to_string(),
            size: size.to_string(),
            direction: direction.to_string(),
            features: existing_features,
        };
        msgs.lock()
            .unwrap()
            .push_back("info--3--typography--change".to_string());
    }
    match write_user_settings(&state, &clients) {
        Ok(_) => {}
        Err(e) => {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!("Could not write out user settings: {}", e)),
            )
        }
    }
    ok_ok_json_response()
}
