use crate::static_vars::NET_IS_ENABLED;
use crate::structs::RemoteRepoRecord;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::response::{
    not_ok_json_response, not_ok_offline_json_response, ok_json_response,
};
use rocket::get;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use serde_json::Value;
use std::sync::atomic::Ordering;

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
///     "released": "2024-11-15T11:06:59Z",
///     "clone_url": "https://git.door43.org/BurritoTruck/fr_psle.git",
///     "updated_at": "2024-11-15T11:06:59Z"
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
        return not_ok_offline_json_response();
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
                        released: match latest["released"].as_str() {
                            Some(s) => s.to_string(),
                            _ => "".to_string(),
                        },
                        updated_at: json_record["updated_at"].as_str().unwrap().to_string(),
                        clone_url: match json_record["clone_url"].as_str() {
                            Some(s) => s.to_string(),
                            _ => "".to_string(),
                        },
                        latest_zip: match json_record["catalog"].as_object() {
                            Some(ob1) => match ob1["latest"].as_object() {
                                Some(ob) => match ob["zipball_url"].as_str() {
                                    Some(s) => s.to_string(),
                                    _ => "".to_string(),
                                },
                                _ => "".to_string(),
                            },
                            _ => "".to_string(),
                        },
                        metadata_types: match json_record["metadata_type"].as_str() {
                            Some(s) => s.to_string(),
                            None => "".to_string(),
                        },
                        topics: match json_record["topics"].as_array() {
                            Some(s) => s
                                .to_vec()
                                .into_iter()
                                .map(|e: Value| -> String {
                                    e.as_str().expect("topics as str").to_string()
                                })
                                .collect(),
                            None => Vec::new(),
                        },
                        book_codes: match json_record["ingredients"].as_array() {
                            Some(s) => s
                                .to_vec()
                                .into_iter()
                                .filter(|e| e.as_object().expect("object in ingredients filter")["exists"].as_bool().expect("exists bool"))
                                .map(|e: Value| -> String {
                                    e.as_object()
                                        .expect("object in ingredients")
                                        ["identifier"]
                                        .as_str()
                                        .expect("identifier to str")
                                        .to_string()
                                })
                                .collect(),
                            None => Vec::new(),
                        },
                    });
                }
                ok_json_response(serde_json::to_string(&records).unwrap())
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
