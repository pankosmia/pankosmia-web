use crate::static_vars::NET_IS_ENABLED;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, not_ok_offline_json_response,
    ok_ok_json_response,
};
use git2::Repository;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};
use std::path::{Components, PathBuf};
use std::sync::atomic::Ordering;

/// *`GET /clone-repo/<repo_path>`*
///
/// Typically mounted as **`/git/clone-repo/<repo_path>`**
///
/// Makes a local clone of a repo from the given repo path.
#[get("/clone-repo/<repo_path..>")]
pub async fn clone_repo(
    state: &State<AppSettings>,
    repo_path: PathBuf,
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
        match Repository::clone(
            &url,
            format!(
                "{}{}{}{}{}{}{}",
                state.repo_dir.lock().unwrap().clone(),
                os_slash_str(),
                source,
                os_slash_str(),
                org,
                os_slash_str(),
                repo.as_str(),
            ),
        ) {
            Ok(_repo) => ok_ok_json_response(),
            Err(e) => not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!("could not clone repo: {}", e).to_string()),
            ),
        }
    } else {
        not_ok_bad_repo_json_response()
    }
}
