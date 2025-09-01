use crate::static_vars::NET_IS_ENABLED;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, not_ok_offline_json_response,
    ok_ok_json_response,
};
use git2::{AutotagOption, FetchOptions, RemoteUpdateFlags, Repository};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};
use std::path::{Components, PathBuf};
use std::sync::atomic::Ordering;
use std::time::SystemTime;

/// *`GET /fetch-repo/<repo_path>`*
///
/// Typically mounted as **`/git/fetch-repo/<repo_path>`**
///
/// Fetches for a repo.
#[get("/fetch-repo/<repo_path..>")]
pub async fn fetch_repo(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    if !NET_IS_ENABLED.load(Ordering::Relaxed) {
        return not_ok_offline_json_response();
    }
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let repo_path_string = format!(
            "{}{}{}",
            state.repo_dir.lock().unwrap().clone(),
            os_slash_str(),
            &repo_path.display().to_string().clone()
        );
        match Repository::open(&repo_path_string) {
            Ok(repo) => {
                let remote_name = "origin";
                let mut remote = repo
                    .find_remote(remote_name)
                    .or_else(|_| repo.remote_anonymous(remote_name))
                    .expect("could not find remote");
                let mut fo = FetchOptions::new();
                match remote.download(&[] as &[&str], Some(&mut fo)) {
                    Ok(_) => {}
                    Err(e) => {
                        return not_ok_json_response(
                            Status::InternalServerError,
                            make_bad_json_data_response(
                                format!("could not fetch repo: {}", e).to_string(),
                            ),
                        )
                    }
                };
                remote.disconnect().expect("could not disconnect remote");
                remote
                    .update_tips(
                        None,
                        RemoteUpdateFlags::UPDATE_FETCHHEAD,
                        AutotagOption::Unspecified,
                        None,
                    )
                    .expect("could not update tips");
                let metadata_path =
                    format!("{}{}metadata.json", &repo_path_string, os_slash_str(),);
                std::fs::File::open(&metadata_path)
                    .expect("Could not open metadata file")
                    .set_modified(SystemTime::now())
                    .expect("Could not set timestamp of metadata file");
                ok_ok_json_response()
            }
            Err(e) => not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("could not open repo: {}", e).to_string()),
            ),
        }
    } else {
        not_ok_bad_repo_json_response()
    }
}
