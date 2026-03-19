use crate::static_vars::NET_IS_ENABLED;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, not_ok_offline_json_response,
    ok_ok_json_response,
};
use git2::{Repository, build::RepoBuilder, FetchOptions, AutotagOption};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{post, State};
use std::path::{Components, PathBuf,Path};
use std::sync::atomic::Ordering;

/// *`POST /clone-repo/<repo_path>?<branch>`*
///
/// Typically mounted as **`/git/clone-repo/<repo_path>?<branch>`**
///
/// Makes a local clone of a repo from the given repo path.
#[post("/clone-repo/<repo_path..>?<branch>")]
pub async fn clone_repo(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    branch: Option<String>,
) -> status::Custom<(ContentType, String)> {
    if !NET_IS_ENABLED.load(Ordering::Relaxed) {
        return not_ok_offline_json_response();
    }
    let mut path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let source = path_components
            .next()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap();
        let org = path_components
            .next()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap();
        let mut repo = path_components
            .next()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();
        if repo.ends_with(".git") {
            let repo_vec = repo.split(".").collect::<Vec<&str>>();
            let short_repo = &repo_vec[0..repo_vec.len() - 1];
            let short_repo_string = short_repo.join("/");
            repo = short_repo_string.as_str().to_owned();
        }
        let url = "https://".to_string() + &repo_path.display().to_string().replace("\\", "/");
                let local_path_str = format!(
            "{}{}{}{}{}{}{}",
            state.repo_dir.lock().unwrap().clone(),
            os_slash_str(),
            source,
            os_slash_str(),
            org,
            os_slash_str(),
            repo.as_str(),
        );
        let local_path = Path::new(&local_path_str);

        if let Some(branch) = &branch {
            let mut fetch_opts = FetchOptions::new();
            fetch_opts.download_tags(AutotagOption::All);
            fetch_opts.depth(1);
            let mut builder = RepoBuilder::new();
            builder.fetch_options(fetch_opts);
            builder.branch(branch);


            match builder.clone(&url, local_path) {
                Ok(_) => ok_ok_json_response(),
                Err(e) => not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(format!("could not clone branch {}: {}", branch, e)),
                ),
            }
        } else {
            match Repository::clone(&url, local_path) {
                Ok(_) => ok_ok_json_response(),
                Err(e) => not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(format!("could not clone repo: {}", e)),
                ),
            }
        }
    } else {
        not_ok_bad_repo_json_response()
    }
}
