use std::path::PathBuf;
use std::sync::atomic::Ordering;
use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::{status, Redirect};
use uuid::Uuid;
use crate::static_vars::NET_IS_ENABLED;
use crate::structs::{AppSettings, AuthRequest, ContentOrRedirect};
use crate::utils::json_responses::make_bad_json_data_response;

/// *`GET /login/<auth_key>/<redir_path..>`*
///
/// Typically mounted as **`/gitea/login/<auth_key>/<redir_path..>`**
///
/// Initiates login to a remote server, which may include redirection to that server's login pages.
#[get("/login/<token_key>/<redir_path..>")]
pub fn gitea_proxy_login(
    state: &State<AppSettings>,
    token_key: String,
    redir_path: PathBuf,
) -> ContentOrRedirect {
    if !NET_IS_ENABLED.load(Ordering::Relaxed) {
        return ContentOrRedirect::Content(
            status::Custom(
                Status::Unauthorized,
                (
                    ContentType::JSON,
                    make_bad_json_data_response("offline mode".to_string()),
                ),
            )
        );
    }
    if !state.gitea_endpoints.contains_key(&token_key) {
        return ContentOrRedirect::Content(
            status::Custom(
                Status::BadRequest,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!(
                        "Unknown GITEA endpoint name: {}",
                        token_key
                    )),
                ),
            )
        );
    }
    // Remove any existing token
    state
        .auth_tokens
        .lock()
        .unwrap()
        .remove(&token_key);
    // Store request info
    let code = Uuid::new_v4().to_string();
    let mut auth_requests = state
        .auth_requests
        .lock()
        .unwrap();
    auth_requests.remove(&token_key);
    auth_requests.insert(
        token_key.clone(),
        AuthRequest {
            code: code.clone(),
            redirect_uri: redir_path.display().to_string(),
            timestamp: std::time::SystemTime::now(),
        },
    );
    // Do redirect
    ContentOrRedirect::Redirect(
        Redirect::to(
            format!("{}/auth?client_code={}&redir_path=%2F", state.gitea_endpoints[&token_key].clone(), &code)
        )
    )
}
