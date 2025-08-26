use crate::static_vars::NET_IS_ENABLED;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, not_ok_offline_json_response,
    ok_ok_json_response,
};
use git2::{Cred, RemoteCallbacks, Repository};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{post, State};
use std::env;
use std::path::{Components, PathBuf};
use std::sync::atomic::Ordering;
use crate::utils::paths::home_dir_string;

#[derive(Deserialize)]
pub struct PushForm {
    remote: String,
    cred_type: String,
    pass_key: Option<String>,
}
/// *`POST /push/<remote>/<repo_path>`*
///
/// Typically mounted as **`/push/<remote>/<repo_path>`**
///
/// Push to remote from the given repo path. In the JSON body,
/// - cred_type is the type of SSH key, eg 'rsa'
/// - pass_key is the optional passkey for the SSH key
#[post("/push/<repo_path..>", format = "json", data = "<json_form>")]
pub async fn push_repo(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    json_form: Json<PushForm>,
) -> status::Custom<(ContentType, String)> {
    if !NET_IS_ENABLED.load(Ordering::Relaxed) {
        return not_ok_offline_json_response();
    }
    let git_credentials_callback = |_user: &str,
                                    _user_from_url: Option<&str>,
                                    _cred: git2::CredentialType|
     -> Result<Cred, git2::Error> {
        let system_user = "git".to_string();
        let user = _user_from_url.unwrap_or(system_user.as_str());
        let cred = match Cred::ssh_key(
            user,
            Some(std::path::Path::new(&format!(
                "{}{}.ssh{}id_{}.pub",
                home_dir_string(),
                os_slash_str(),
                os_slash_str(),
                &json_form.cred_type
            ))),
            std::path::Path::new(&format!(
                "{}{}.ssh{}id_{}",
                home_dir_string(),
                os_slash_str(),
                os_slash_str(),
                &json_form.cred_type
            )),
            match &json_form.pass_key {
                Some(s) => Some(s.as_str()),
                None => None,
            },
        ) {
            Ok(c) => {
                // println!("Cred created successfully!");
                Ok(c)
            }
            Err(e) => {
                println!("Could not create cred: {}", e);
                Err(e)
            }
        };
        cred
    };

    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let repo_path_string = format!(
            "{}{}{}",
            state.repo_dir.lock().unwrap().clone(),
            os_slash_str(),
            &repo_path.display().to_string()
        );
        match Repository::open(repo_path_string) {
            Ok(repo) => {
                let mut remote_object = match repo.find_remote(&json_form.remote.as_str()) {
                    Ok(ro) => ro,
                    Err(e) => {
                        return not_ok_json_response(
                            Status::BadRequest,
                            make_bad_json_data_response(format!(
                                "Could not find remote {}: {}",
                                &json_form.remote, e
                            )),
                        )
                    }
                };
                let head = repo.head().expect("Could not locate head");
                let head_branch_name = head.name().expect("Could not get branch name from head");
                let mut remote_callbacks = RemoteCallbacks::new();
                remote_callbacks.credentials(git_credentials_callback);
                let mut push_options = git2::PushOptions::new();
                push_options.remote_callbacks(remote_callbacks);
                match remote_object.push::<&str>(&[head_branch_name], Some(&mut push_options)) {
                    Ok(_) => ok_ok_json_response(),
                    Err(e) => not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!("Could not push repo: {}", e)),
                    ),
                }
            }
            Err(e) => not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not open repo: {}", e)),
            ),
        }
    } else {
        not_ok_bad_repo_json_response()
    }
}
