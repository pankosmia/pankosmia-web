use crate::structs::AppSettings;
use crate::utils::burrito::{copy_gitignore_template, language_name_from_code};
use crate::utils::files::paths_to_new_burrito;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_ok_json_response};
use crate::utils::time::utc_now_timestamp_string;
use crate::utils::git::{init_repo, add_and_commit_repo};
use copy_dir::copy_dir;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{post, FromForm, State};
use serde_json::json;

#[derive(FromForm, Deserialize)]
pub struct NewObsContentForm {
    pub content_name: String,
    pub content_abbr: String,
    pub copyright: Option<String>,
    pub content_language_code: String,
    pub content_language_name: Option<String>,
    pub branch_name: Option<String>,
}

/// *`POST /new-obs-resource`*
///
/// Typically mounted as **`/git/new-obs-resource`**
///
/// Creates a new, local obs repo. It requires the following fields as a JSON body:
/// - content_name (string)
/// - content_abbr (string)
/// - content_language_code (string)
/// - content_language_name (null or string)
/// - branch_name (null or string)
#[post("/new-obs-resource", format = "json", data = "<json_form>")]
pub fn new_obs_resource_repo(
    state: &State<AppSettings>,
    json_form: Json<NewObsContentForm>,
) -> status::Custom<(ContentType, String)> {
    // Check template type exists
    let path_to_template = format!(
        "{}{}templates{}content_templates{}text_stories{}metadata.json",
        &state.app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
    );
    if !std::path::Path::new(&path_to_template).is_file() {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!(
                "Metadata template {} not found",
                path_to_template
            )),
        );
    }

    let language_name = match language_name_from_code(
        &state.app_resources_dir,
        json_form.content_language_code.clone(),
        json_form.content_language_name.clone(),
    ) {
        Ok(v) => v,
        Err(e) => {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!(
                    "Unable to find language name: {}: {}",
                    &json_form.content_language_code, e
                )),
            );
        }
    };

    let (path_to_new_repo_parent, path_to_new_repo) =
        match paths_to_new_burrito(&state.repo_dir.lock().unwrap(), &json_form.content_abbr) {
            Ok(tup) => tup,
            Err(e) => {
                return not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(format!("Could not get new repo paths: {}", e)),
                )
            }
        };

    // Make parents?
    match std::fs::create_dir_all(path_to_new_repo_parent) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not create local content directories: {}",
                    e
                )),
            )
        }
    }
    let new_repo = match init_repo(&path_to_new_repo, json_form.branch_name.clone()) {
        Ok(r) => r,
        Err(e) => {
            return status::Custom(
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!(
                        "{}",
                        e
                    )),
                ),
            )
        }
    };
    // Copy ingredients dir
    let path_to_ingredients_template = format!(
        "{}{}templates{}content_templates{}text_stories{}ingredients",
        &state.app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
    );
    let path_to_ingredients = format!("{}{}ingredients", path_to_new_repo, os_slash_str(),);
    match copy_dir(&path_to_ingredients_template, &path_to_ingredients) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not copy ingredients directory for repo: {}",
                    e
                )),
            )
        }
    }

    match copy_gitignore_template(&state.app_resources_dir, &path_to_new_repo) {
        Ok(_) => {},
        Err(e) => return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("{}", e)),
            )
    };

    // Read and customize metadata
    let mut metadata_string = match std::fs::read_to_string(&path_to_template) {
        Ok(v) => v,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not load metadata template as string: {}",
                    e
                )),
            )
        }
    };
    let now_time = utc_now_timestamp_string();
    let language_json = json!(
        {
            "tag": &json_form.content_language_code.clone(),
            "name": {
                "en": &language_name,
        }
        }
    );
    metadata_string = metadata_string
        .replace("%%ABBR%%", json_form.content_abbr.as_str())
        .replace("%%CONTENT_NAME%%", json_form.content_name.as_str())
        .replace("%%COPYRIGHT%%", json_form.copyright.clone().unwrap_or("unspecified".to_string()).as_str())
        .replace("%%CREATED_TIMESTAMP%%", now_time.to_string().as_str())
        .replace(
            "%%LANGUAGE%%",
            serde_json::to_string(&language_json)
                .expect("language json")
                .as_str(),
        );

    // Write metadata
    let path_to_repo_metadata = format!("{}{}metadata.json", &path_to_new_repo, os_slash_str());
    match std::fs::write(path_to_repo_metadata, metadata_string) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not write metadata template to repo: {}",
                    e
                )),
            )
        }
    }
    match add_and_commit_repo(new_repo, &"Initial Commit".to_string()) {
        Ok(_) => {},
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "{}",
                    e
                )),
            )
        }
    };
    ok_ok_json_response()
}
