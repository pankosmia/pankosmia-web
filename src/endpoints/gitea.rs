use std::path::PathBuf;
use std::sync::atomic::Ordering;
use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::{status, Redirect};
use serde_json::Value;
use uuid::Uuid;
use crate::static_vars::NET_IS_ENABLED;
use crate::structs::{AppSettings, AuthRequest, ContentOrRedirect, RemoteRepoRecord};
use crate::utils::json_responses::make_bad_json_data_response;

/// *```GET /endpoints```*
///
/// Typically mounted as **`/gitea/endpoints`**
///
/// Returns an object containing gitea gateway keys and urls.
///
/// ```text
/// {"xenizo_syllogos":"http://xenizo.fr:8089"}
/// ```
#[get("/endpoints")]
pub fn get_gitea_endpoints(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    status::Custom(
        Status::Ok,
        (ContentType::JSON, serde_json::to_string(&state.gitea_endpoints).unwrap()),
    )
}

/// *`GET /remote-repos/<gitea_server>/<gitea_org>`*
///
/// Typically mounted as **`/gitea/remote-repos/<gitea_server>/<gitea_org>`**
///
/// Returns an object containing repo info for a given gitea organization.
///
/// ```text
/// [
///   {
///     "name": "fr_psle",
///     "abbreviation": "psle",
///     "description": "Une traduction littéralement plus simple",
///     "avatar_url": "https://git.door43.org/repo-avatars/f052d1bba37e57e0ec56bd68b6274290310d3bfc392cd4534b1d4a0814cccb36",
///     "flavor": "textTranslation",
///     "flavor_type": "scripture",
///     "language_code": "fr",
///     "script_direction": "ltr",
///     "branch_or_tag": "master",
///     "clone_url": "2024-11-15T11:06:59Z"
///   },
///   ...
/// ]
/// ```
#[get("/remote-repos/<gitea_server>/<gitea_org>")]
pub fn gitea_remote_repos(
    gitea_server: &str,
    gitea_org: &str,
) -> status::Custom<(ContentType, String)> {
    if !NET_IS_ENABLED.load(Ordering::Relaxed) {
        return status::Custom(
            Status::Unauthorized,
            (
                ContentType::JSON,
                make_bad_json_data_response("offline mode".to_string()),
            ),
        );
    }
    let gitea_path = format!("https://{}/api/v1/orgs/{}/repos", gitea_server, gitea_org);
    match ureq::get(gitea_path.as_str()).call() {
        Ok(r) => match r.into_json::<Value>() {
            Ok(j) => {
                let mut records: Vec<RemoteRepoRecord> = Vec::new();
                for json_record in j.as_array().unwrap() {
                    let latest = &json_record["catalog"]["latest"];
                    records.push(RemoteRepoRecord {
                        name: json_record["name"].as_str().unwrap().to_string(),
                        abbreviation: json_record["abbreviation"].as_str().unwrap().to_string(),
                        description: json_record["description"].as_str().unwrap().to_string(),
                        avatar_url: json_record["avatar_url"].as_str().unwrap().to_string(),
                        flavor: json_record["flavor"].as_str().unwrap().to_string(),
                        flavor_type: json_record["flavor_type"].as_str().unwrap().to_string(),
                        language_code: json_record["language"].as_str().unwrap().to_string(),
                        script_direction: json_record["language_direction"]
                            .as_str()
                            .unwrap()
                            .to_string(),
                        branch_or_tag: match latest["branch_or_tag_name"].as_str() {
                            Some(s) => s.to_string(),
                            _ => "".to_string(),
                        },
                        clone_url: match latest["released"].as_str() {
                            Some(s) => s.to_string(),
                            _ => "".to_string(),
                        },
                    });
                }
                status::Custom(
                    Status::Ok,
                    (ContentType::JSON, serde_json::to_string(&records).unwrap()),
                )
            }
            Err(e) => {
                return status::Custom(
                    Status::InternalServerError,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response(format!(
                            "could not serve GITEA server response as JSON string: {}",
                            e
                        )),
                    ),
                )
            }
        },
        Err(e) => status::Custom(
            Status::BadGateway,
            (
                ContentType::JSON,
                make_bad_json_data_response(
                    format!("could not read from GITEA server: {}", e).to_string(),
                ),
            ),
        ),
    }
}

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
            timestamp: std::time::SystemTime::now()
        }
    );
    // Do redirect
    ContentOrRedirect::Redirect(
        Redirect::to(
            format!("{}/auth?client_code={}&redir_path=%2F", state.gitea_endpoints[&token_key].clone(), &code)
        )
    )
}

/// *`GET /logout/<auth_key>`* (uses cookie with key `<auth_key>_code`)
///
/// Typically mounted as **`/gitea/logout/<auth_key>`**
///
/// Logs out of a remote server.
#[get("/logout/<token_key>")]
pub fn gitea_proxy_logout(
    state: &State<AppSettings>,
    token_key: String
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
    // Logout of proxy server
    let logout_url = format!("{}/logout", state.gitea_endpoints[&token_key].clone());
    println!("{}", logout_url);
    match ureq::get(logout_url.as_str()).call() {
        Ok(_) => {
            // Remove any existing token
            state
                .auth_tokens
                .lock()
                .unwrap()
                .remove(&token_key);
            // Do redirect
            ContentOrRedirect::Redirect(
                Redirect::to("/")
            )
        },
        Err(e) => ContentOrRedirect::Content(
            status::Custom(
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!(
                        "Error on logout from proxy {}: {}",
                        token_key,
                        e
                    )),
                ),
            )
        )
    }
}
