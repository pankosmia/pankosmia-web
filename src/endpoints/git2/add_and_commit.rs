use crate::structs::{AppSettings, BurritoMetadata};
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_ok_json_response,
};
use crate::utils::time::utc_now_timestamp_string;
use git2::Repository;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{post, State};
use std::path::{Components, PathBuf};
use serde_json::{Map, Value};

/// *`POST /add-and-commit/<repo_path>`*
///
/// Typically mounted as **`/git/add-and-commit/<repo_path>`**
///
/// Adds and commits modified files for a given repo.

#[derive(Deserialize)]
pub struct AddCommitForm {
    commit_message: String,
}

#[post("/add-and-commit/<repo_path..>", format = "json", data = "<json_form>")]
pub async fn add_and_commit(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    json_form: Json<AddCommitForm>,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let repo_path_string = format!(
            "{}{}{}",
            state.repo_dir.lock().unwrap().clone(),
            os_slash_str(),
            &repo_path.display().to_string().clone()
        );
        // Read repo metadata
        let path_to_repo_metadata = format!("{}{}metadata.json", repo_path_string, os_slash_str(),);
        let metadata_string = match std::fs::read_to_string(&path_to_repo_metadata) {
            Ok(v) => v,
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!(
                        "Could not load metadata as string: {}",
                        e
                    )),
                )
            }
        };
        // Make struct from metadata
        let mut metadata_struct: BurritoMetadata = match serde_json::from_str(&metadata_string) {
            Ok(v) => v,
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Could not parse metadata: {}", e)),
                );
            }        // Write metadata

        };
        // Update timestamps & revision
        let now_time = utc_now_timestamp_string();
        let mut meta_obj: Map<String, Value> = metadata_struct.meta.as_object().expect("meta object").clone();
        meta_obj.insert("dateCreated".to_string(), Value::String(now_time));
        metadata_struct.meta = serde_json::to_value(meta_obj).expect("meta map to value");
        // Write metadata
        let new_metadata_string = serde_json::to_string(&metadata_struct).expect("new metadata string");
        match std::fs::write(path_to_repo_metadata, new_metadata_string) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not write updated metadata to repo: {}",
                    e
                )),
            )
        }
    }
        // Git - open, add and commit repo
        let result = match Repository::open(repo_path_string) {
            Ok(repo) => {
                repo.index()
                    .expect("repo index")
                    .add_all(&["."], git2::IndexAddOption::DEFAULT, None)
                    .expect("add all");
                repo.index()
                    .expect("repo index 2")
                    .write()
                    .expect("repo index write");
                let mut index = repo.index().expect("repo index 3");
                let oid = index.write_tree().expect("write tree");
                let signature = repo.signature().expect("signature");
                let parent_commit = repo
                    .head()
                    .expect("repo head")
                    .peel_to_commit()
                    .expect("peel to commit");
                let tree = repo.find_tree(oid).expect("find tree");
                repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    json_form.commit_message.as_str(),
                    &tree,
                    &[&parent_commit],
                )
                .expect("commit");
                ok_ok_json_response()
            }
            Err(e) => not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("could not open repo: {}", e).to_string()),
            ),
        };
        result
    } else {
        not_ok_bad_repo_json_response()
    }
}
