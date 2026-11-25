use crate::static_vars::NET_IS_ENABLED;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::response::{
    not_ok_json_response, not_ok_offline_json_response, ok_json_response,
};
use rocket::get;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use serde_json::Value;
use std::sync::atomic::Ordering;

/// *`GET /remote-repos/<gitea_server>/<gitea_org>/<gitea_repo>/branches`*
///
/// Typically mounted as **`/api/gitea/remote-repos/<gitea_server>/<gitea_org>/<gitea_repo>/branches`**
///
/// Returns an array containing branch names for a given repo.
#[get("/remote-repos/<gitea_server>/<gitea_org>/<gitea_repo>/branches")]
pub fn gitea_remote_repo_branches(
    gitea_server: &str,
    gitea_org: &str,
    gitea_repo: &str
) -> status::Custom<(ContentType, String)> {
    if !NET_IS_ENABLED.load(Ordering::Relaxed) {
        return not_ok_offline_json_response();
    }
    let gitea_path = format!("https://{}/api/v1/repos/{}/{}/branches", gitea_server, gitea_org, gitea_repo);
    match ureq::get(gitea_path.as_str()).call() {
        Ok(r) => match r.into_body().read_json::<Value>() {
            Ok(j) => {
                let mut branches: Vec<String> = Vec::new();
                for json_record in j.as_array().expect("response body as array") {
                    let record_name = json_record["name"].as_str().expect("record name as string");
                    branches.push(record_name.to_string());
                }
                ok_json_response(serde_json::to_string(&branches).unwrap())
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
