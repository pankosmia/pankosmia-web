use std::path::{Components, PathBuf};
use std::sync::atomic::Ordering;
use git2::Repository;
use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::static_vars::NET_IS_ENABLED;
use crate::structs::AppSettings;
use crate::utils::json_responses::{make_bad_json_data_response, make_good_json_data_response};
use crate::utils::paths::{check_path_components, os_slash_str};

/// *`GET /fetch-repo/<repo_path>`*
///
/// Typically mounted as **`/git/fetch-repo/<repo_path>`**
///
/// Makes a local clone of a repo from the given repo path.
#[get("/fetch-repo/<repo_path..>")]
pub async fn fetch_repo(
    state: &State<AppSettings>,
    repo_path: PathBuf,
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
            state.repo_dir.lock().unwrap().clone()
                + os_slash_str()
                + source
                + os_slash_str()
                + org
                + os_slash_str()
                + repo.as_str(),
        ) {
            Ok(_repo) => status::Custom(
                Status::Ok,
                (
                    ContentType::JSON,
                    make_good_json_data_response("ok".to_string()),
                ),
            ),
            Err(e) => {
                println!("Error:{}", e);
                status::Custom(
                    Status::BadRequest,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response(
                            format!("could not clone repo: {}", e).to_string(),
                        ),
                    ),
                )
            }
        }
    } else {
        status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response("bad repo path".to_string()),
            ),
        )
    }
}