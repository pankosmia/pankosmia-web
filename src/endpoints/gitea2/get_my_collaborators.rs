use std::sync::atomic::Ordering;
use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::static_vars::NET_IS_ENABLED;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;

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
    project: String
) -> status::Custom<(ContentType, String)> {
    // Require Net
    if !NET_IS_ENABLED.load(Ordering::Relaxed) {
        return
            status::Custom(
                Status::Unauthorized,
                (
                    ContentType::JSON,
                    make_bad_json_data_response("offline mode".to_string()),
                ),
            );
    }
    // Proxy must exist
    if !state.gitea_endpoints.contains_key(&proxy) {
        return
            status::Custom(
                Status::BadRequest,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!(
                        "Unknown GITEA endpoint name: {}",
                        proxy
                    )),
                ),
            );
    }
    // There must be a code for the proxy
    let auth_tokens = state
        .auth_tokens
        .lock()
        .unwrap()
        .clone();
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
            Ok(r) => {
                status::Custom(
                    Status::Ok,
                    (ContentType::JSON, r.into_string().unwrap()),
                )
            }
            Err(e) => status::Custom(
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!(
                        "Error from proxy {}: {}",
                        proxy,
                        e
                    )),
                ),
            )
        }
    } else {
        status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response(format!(
                    "Could not find record for proxy '{}'",
                    proxy
                )),
            ),
        )
    }
}
