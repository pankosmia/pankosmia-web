use crate::structs::AppSettings;
use crate::structs::MetadataSummary;
use crate::utils::burrito::summary_metadata_from_file;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};

/// *`GET /metadata/summaries?<org>`*
///
/// Typically mounted as **`/burrito/metadata/summaries?<org>`**
///
/// Returns a JSON array of local repo metadata objects, optionally only for a given org.

#[get("/metadata/summaries?<org>")]
pub fn summary_metadatas(
    state: &State<AppSettings>,
    org: Option<String>,
) -> status::Custom<(ContentType, String)> {
    let root_path = state.repo_dir.lock().unwrap().clone();
    let server_paths = std::fs::read_dir(root_path).unwrap();
    let mut repos: std::collections::BTreeMap<String, MetadataSummary> =
        std::collections::BTreeMap::new();
    for server_path in server_paths {
        let uw_server_path_ob = server_path.unwrap().path();
        let uw_server_path_ob2 = uw_server_path_ob.clone();
        let server_leaf = uw_server_path_ob2.file_name().unwrap();
        if !std::path::Path::new(&uw_server_path_ob).is_dir() {
            println!("Skipping server non-dir {}", server_leaf.to_string_lossy());
            continue;
        }
        for org_path in std::fs::read_dir(uw_server_path_ob).unwrap() {
            let uw_org_path_ob = org_path.unwrap().path();
            let uw_org_path_ob2 = uw_org_path_ob.clone();
            let org_leaf = uw_org_path_ob2.file_name().unwrap();
            let server_org = format!(
                "{}/{}",
                server_leaf.to_str().unwrap(),
                org_leaf.to_str().unwrap()
            );
            match org.clone() {
                Some(o) => {
                    if o != server_org {
                        continue;
                    }
                }
                _ => {
                    if server_org == "_local_/_quarantine_" {
                        continue;
                    }
                }
            }
            if !std::path::Path::new(&uw_org_path_ob).is_dir() {
                println!("Skipping org non-dir {}", &server_org);
                continue;
            }
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
                    Err(e) => MetadataSummary {
                        name: "? Bad Metadata JSON ?".to_string(),
                        description: "?".to_string(),
                        abbreviation: "?".to_string(),
                        generated_date: "?".to_string(),
                        flavor_type: "?".to_string(),
                        flavor: "?".to_string(),
                        language_code: "?".to_string(),
                        script_direction: "?".to_string(),
                        book_codes: vec![],
                    },
                };

                repos.insert(repo_url_string, summary);
            }
        }
    }
    ok_json_response(serde_json::to_string(&repos).unwrap())
}
