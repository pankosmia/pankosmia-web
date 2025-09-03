use crate::structs::AppSettings;
use crate::utils::response::ok_json_response;
use rocket::http::{ContentType};
use rocket::response::status;
use rocket::{get, State};

/// *`GET /list-local-repos`*
///
/// Typically mounted as **`/git/list-local-repos`**
///
/// Returns a JSON array of local repo paths.
///
/// `["git.door43.org/BurritoTruck/fr_psle"]`
#[get("/list-local-repos")]
pub fn list_local_repos(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let root_path = state.repo_dir.lock().unwrap().clone();
    let server_paths = std::fs::read_dir(root_path).unwrap();
    let mut repos: Vec<String> = Vec::new();
    for server_path in server_paths {
        let uw_server_path_ob = server_path.unwrap().path();
        let uw_server_path_ob2 = uw_server_path_ob.clone();
        let server_leaf = uw_server_path_ob2.file_name().unwrap();
        if server_leaf.to_str().unwrap().starts_with(".") {
            println!("Skipping server . file or dir {}", &server_leaf.to_str().unwrap());
            continue;
        }
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
            if org_leaf.to_str().unwrap().starts_with(".") {
                println!("Skipping org . file or dir {}", &server_org);
                continue;
            }
            if !std::path::Path::new(&uw_org_path_ob).is_dir() {
                println!("Skipping org non-dir {}", &server_org);
                continue;
            }
            if server_org == "_local_/_quarantine_" {continue};
            if server_org == "_local_/_archive_" {continue};
            if server_org == "_local_/_updates_" {continue};
            for repo_path in std::fs::read_dir(uw_org_path_ob).unwrap() {
                let uw_repo_path_ob = repo_path.unwrap().path();
                let repo_leaf = uw_repo_path_ob.file_name().unwrap();
                let repo_url_string = format!(
                    "{}/{}/{}",
                    server_leaf.to_str().unwrap(),
                    org_leaf.to_str().unwrap(),
                    repo_leaf.to_str().unwrap()
                );
                if repo_leaf.to_str().unwrap().starts_with(".") {
                    println!("Skipping repo . file or dir {}", &repo_leaf.to_str().unwrap());
                    continue;
                }
                if !std::path::Path::new(&uw_repo_path_ob).is_dir() {
                    println!("Skipping repo non-dir {}", &repo_url_string);
                    continue;
                }
                repos.push(repo_url_string);
            }
        }
    }
    let quoted_repos: Vec<String> = repos
        .into_iter()
        .map(|str: String| format!("{}", str))
        .collect();
    ok_json_response(serde_json::to_string(&quoted_repos).unwrap())
}
