use crate::static_vars::NET_IS_ENABLED;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::response::{not_ok_json_response, ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};
use std::sync::atomic::Ordering;

/// *`/my-collaborators/<proxy>/<organization>/<project>`*
///
/// Typically mounted as **`/my-collaborators/<proxy>/<organization>/<project>`**
///
/// Returns an array containing the current selected UI languages.
///
/// `[{"id": 12345, "login": "Martine Dupont"}]`
#[get("/my-collaborators/<proxy>/<organization>/<project>")]
pub fn get_my_collaborators(
    state: &State<AppSettings>,
    proxy: String,
    organization: String,
    project: String,
) -> status::Custom<(ContentType, String)> {
    // Require Net
    if !NET_IS_ENABLED.load(Ordering::Relaxed) {
        return not_ok_json_response(
            Status::Unauthorized,
            make_bad_json_data_response("offline mode".to_string()),
        );
    }
    // Proxy must exist
    if !state.gitea_endpoints.contains_key(&proxy) {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!("Unknown GITEA endpoint name: {}", proxy)),
        );
    }
    // There must be a code for the proxy
    let auth_tokens = state.auth_tokens.lock().unwrap().clone();
    if auth_tokens.contains_key(&proxy) {
        // We have a code
        let code = &auth_tokens[&proxy];
        let collab_url = format!(
            "{}/repos-collaborators?organisation_name={}&project_name={}&client_code={}",
            state.gitea_endpoints[&proxy].clone(),
            &organization,
            &project,
            code
        );
        match ureq::get(collab_url.as_str()).call() {
            Ok(r) => ok_json_response(r.into_string().unwrap()),
            Err(e) => not_ok_json_response(
                Status::BadGateway,
                make_bad_json_data_response(format!("Error from proxy {}: {}", proxy, e)),
            ),
        }
    } else {
        not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!("Could not find record for proxy '{}'", proxy)),
        )
    }
}
