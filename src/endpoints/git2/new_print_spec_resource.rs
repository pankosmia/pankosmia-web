use crate::structs::AppSettings;
use crate::utils::burrito::{copy_gitignore_template, language_name_from_code};
use crate::utils::files::paths_to_new_burrito;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_ok_json_response};
use crate::utils::time::utc_now_timestamp_string;
use crate::utils::git::{init_repo, add_and_commit_repo};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{post, FromForm, State};
use serde_json;
use serde_json::json;

/// *`POST /new-print-spec-resource`*
///
/// Typically mounted as **`/git/new-print-spec-resource`**
///
/// Creates a new, local *x-printspec* repo. It requires the following fields as a JSON body:
/// - content_name (string)
/// - content_abbr (string)
/// - copyright (string)
/// - content_language_code
/// - content_language_name (null or string)
/// - branch_name(null or string)
/// - copyright(string)
/// - spec (string)

#[derive(FromForm, Deserialize)]
pub struct NewPrintSpecContentForm {
    pub content_name: String,
    pub content_abbr: String,
    pub copyright: Option<String>,
    pub content_language_code: String,
    pub content_language_name: Option<String>,
    pub branch_name: Option<String>,
    pub spec: Option<String>,
}

#[post("/new-print-spec-resource", format = "json", data = "<json_form>")]
pub fn new_print_spec_resource_repo(
    state: &State<AppSettings>,
    json_form: Json<NewPrintSpecContentForm>,
) -> status::Custom<(ContentType, String)> {
    // Check template type exists
    let path_to_template = format!(
        "{}{}templates{}content_templates{}x-printspec{}metadata.json",
        &state.app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
    );
    if !std::path::Path::new(&path_to_template).is_file() {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!("Metadata template not found")),
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
    // Make ingredients dir
    let path_to_ingredients = format!("{}{}ingredients", path_to_new_repo, os_slash_str(),);
    match std::fs::create_dir(&path_to_ingredients) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not create ingredients directory for repo: {}",
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

    // Use default spec template or make supplied spec
    let path_to_repo_spec = format!(
        "{}{}ingredients{}plan.json",
        path_to_new_repo,
        os_slash_str(),
        os_slash_str()
    );
    let spec_string = match json_form.spec.clone() {
        Some(s) => match serde_json::from_str(&s) {
            Ok(sv) => sv,
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!(
                        "Could not read supplied spec as JSON: {}",
                        e
                    )),
                )
            }
        },
        None => {
            let path_to_spec_template = format!(
                "{}{}templates{}content_templates{}x-printspec{}spec.json",
                &state.app_resources_dir,
                os_slash_str(),
                os_slash_str(),
                os_slash_str(),
                os_slash_str(),
            );
            match std::fs::read_to_string(&path_to_spec_template) {
                Ok(v) => v,
                Err(e) => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!(
                            "Could not load print spec template as string: {}",
                            e
                        )),
                    )
                }
            }
        }
    };

    match std::fs::write(path_to_repo_spec, &spec_string) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not write print spec to repo: {}", e)),
            )
        }
    }

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
            "tag": &&json_form.content_language_code,
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

    // - add ingredient to metadata
    let ingredient_json = json!(
        {
            "ingredients/spec.json": {
                "checksum": {
                    "md5": format!("{:?}", md5::compute(&spec_string))
                },
                "mimeType": "application/json",
                "size": spec_string.len()
            }
        }
    );

    metadata_string = metadata_string.replace(
        "%%INGREDIENTS%%",
        serde_json::to_string(&ingredient_json).unwrap().as_str(),
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
