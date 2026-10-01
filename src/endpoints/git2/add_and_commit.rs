use crate::structs::{AppSettings, BurritoMetadata};
use crate::utils::burrito::ingredients_metadata_from_files;
use crate::utils::git::add_and_commit_repo;
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
use serde_json::{Map, Value};
use std::path::{Components, PathBuf};
use uuid::Uuid;

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
            state.repo_dir.lock().expect("repo dir lock").clone(),
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
            } // Write metadata
        };
        // Update timestamps & revision
        let now_time = utc_now_timestamp_string();
        // - meta timestamp
        let mut meta_obj: Map<String, Value> = metadata_struct
            .meta
            .as_object()
            .expect("meta object")
            .clone();
        meta_obj.insert("dateCreated".to_string(), Value::String(now_time.clone()));
        metadata_struct.meta = serde_json::to_value(meta_obj).expect("meta map to value");
        // - find identification abbr object
        let mut identification_obj: Map<String, Value> = metadata_struct
            .identification
            .as_object()
            .expect("identification as object")
            .clone();
        let mut primary_obj = identification_obj["primary"]
            .as_object()
            .expect("primary as object")
            .clone();
        let first_primary_org_tuple = primary_obj.iter().next().expect("first org");
        let first_primary_org_key = first_primary_org_tuple.0;
        let mut first_primary_org = first_primary_org_tuple
            .1
            .as_object()
            .expect("org object")
            .clone();
        let first_primary_abbr_tuple = first_primary_org.iter().next().expect("first abbr");
        let first_primary_abbr_key = first_primary_abbr_tuple.0;
        let mut first_primary_abbr = first_primary_abbr_tuple
            .1
            .as_object()
            .expect("abbr object")
            .clone();
        let new_revision = Uuid::new_v4().to_string();
        first_primary_abbr.insert("revision".to_string(), Value::String(new_revision));
        // - identification timestamp
        first_primary_abbr.insert("timestamp".to_string(), Value::String(now_time));
        // Update primary in identification
        first_primary_org.insert(
            first_primary_abbr_key.clone(),
            serde_json::to_value(first_primary_abbr).expect("abbr value"),
        );
        primary_obj.insert(
            first_primary_org_key.clone(),
            serde_json::to_value(first_primary_org).expect("org value"),
        );
        // Update metadata struct
        identification_obj.insert("primary".to_string(), Value::Object(primary_obj));
        metadata_struct.identification =
            serde_json::to_value(identification_obj).expect("new identification");
        // Remake ingredients metadata
        let app_resources_dir = format!("{}", &state.app_resources_dir);
        #[allow(irrefutable_let_patterns)]
        if let mut ingredients = metadata_struct
            .ingredients
            .lock()
            .expect("ingredients lock")
        {
            let new_ingredients = ingredients_metadata_from_files(
                app_resources_dir.clone(),
                repo_path_string.clone(),
            );
            *ingredients = new_ingredients;
        }
        // Write metadata
        let new_metadata_string =
            serde_json::to_string(&metadata_struct).expect("new metadata string");
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
        };
        // Git - open, add and commit repo
        let result = match Repository::open(repo_path_string) {
            Ok(repo) => match add_and_commit_repo(repo, &json_form.commit_message) {
                Ok(_) => ok_ok_json_response(),
                Err(e) => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(
                            format!("could not commit: {}", e).to_string(),
                        ),
                    )
                }
            },
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
