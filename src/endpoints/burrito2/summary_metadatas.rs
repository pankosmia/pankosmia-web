use crate::structs::AppSettings;
use crate::structs::MetadataSummary;
use crate::utils::burrito::summary_metadata_from_file;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{
    not_ok_json_response, ok_json_response,
};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};

/// *`GET /metadata/summaries`*
///
/// Typically mounted as **`/burrito/metadata/summaries`**
///
/// Returns a JSON array of local repo metadata objects.

#[get("/metadata/summaries")]
pub fn summary_metadatas(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let root_path = state.repo_dir.lock().unwrap().clone();
    let server_paths = std::fs::read_dir(root_path).unwrap();
    let mut repos: std::collections::BTreeMap<String, MetadataSummary> = std::collections::BTreeMap::new();
    for server_path in server_paths {
        let uw_server_path_ob = server_path.unwrap().path();
        let uw_server_path_ob2 = uw_server_path_ob.clone();
        let server_leaf = uw_server_path_ob2.file_name().unwrap();
        for org_path in std::fs::read_dir(uw_server_path_ob).unwrap() {
            let uw_org_path_ob = org_path.unwrap().path();
            let uw_org_path_ob2 = uw_org_path_ob.clone();
            let org_leaf = uw_org_path_ob2.file_name().unwrap();
            for repo_path in std::fs::read_dir(uw_org_path_ob).unwrap() {
                let uw_repo_path_ob = repo_path.unwrap().path();
                let repo_leaf = uw_repo_path_ob.file_name().unwrap();
                let repo_url_string = format!(
                    "{}/{}/{}",
                    server_leaf.to_str().unwrap(),
                    org_leaf.to_str().unwrap(),
                    repo_leaf.to_str().unwrap()
                );
                let metadata_path = format!(
                    "{}{}{}{}metadata.json",
                    state.repo_dir.lock().unwrap().clone(),
                    os_slash_str(),
                    &repo_url_string,
                    os_slash_str()
                );
                let summary = match summary_metadata_from_file(metadata_path) {
                    Ok(v) => v,
                    Err(e) => return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(
                            format!("could not extract metadata summary for {}: {}", repo_url_string, e).to_string(),
                        ),
                    )
                };

                repos.insert(repo_url_string, summary);
            }
        }
    }
    ok_json_response(serde_json::to_string(&repos).unwrap())
}
