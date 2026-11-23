use crate::structs::{AppSettings, BurritoMetadata, EditMetadataRecord};
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, check_path_string_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_ok_json_response,
};
use copy_dir::copy_dir;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{post, State};
use std::path::{Components, PathBuf};
use crate::utils::burrito::{destination_parent, edit_metadata};
use rocket::serde::json::Json;

/// *`POST /fork/<repo_path>?target_path=<target_path>&delete_src&add_ignore`*
///
/// Typically mounted as **`/api/git/fork/<repo_path>?target_path=<target_path>&delete_src`**
///
/// Forks a repo to a new location, optionally deleting the source and optionally adding a .gitignore file.
/// 
/// The updated Dublin Core-like metadatais is provided as a JSON payload:
/// 
/// ```{
///    "name": "NEW NAME",
///    "description": "THIS IS A NEW DESCRIPTION",
///    "abbreviation": "NN",
///    "language_code": "x-foo",
///    "language_name": "Green Fooish",
///    "script_direction": "rtl",
///    "copyright": "© The Foo Fairies"
/// }```
#[post("/fork/<repo_path..>?<target_path>&<delete_src>&<add_ignore>", format = "json", data = "<edits>")]
pub async fn fork_repo(
    state: &State<AppSettings>,
    edits: Json<EditMetadataRecord>,
    repo_path: PathBuf,
    target_path: String,
    delete_src: Option<bool>,
    add_ignore: Option<bool>
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone())
        && check_path_string_components(target_path.clone())
    {
        let full_src_path = format!(
            "{}{}{}",
            &state.repo_dir.lock().unwrap(),
            os_slash_str(),
            &repo_path.display().to_string()
        );
        // Src repo must exist
        if !std::path::Path::new(&full_src_path).is_dir() {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("Source repo not found or not a directory".to_string()),
            );
        }
        let full_target_path = format!(
            "{}{}{}",
            &state.repo_dir.lock().unwrap(),
            os_slash_str(),
            &target_path
        );
        // Target repo dir must not exist
        if std::path::Path::new(&full_target_path).is_dir() {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("Target repo already exists".to_string()),
            );
        }
        // Make subdirs if necessary
        let target_parent = destination_parent(full_target_path.clone());
        if !std::path::Path::new(&target_parent).exists() {
            match std::fs::create_dir_all(target_parent) {
                Ok(_) => (),
                Err(e) => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!(
                            "Could not create target parent directories: {}",
                            e
                        )),
                    )
                }
            }
        }
        // copy repo
        match copy_dir(full_src_path.clone(), full_target_path.clone()) {
            Ok(_) => {}
            Err(e) => {
                return not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(format!("could not copy repo: {}", e).to_string()),
                )
            }
        }
        // Edit metadata
        // - Get metadata as string
        let path_to_repo_metadata = format!("{}{}metadata.json", &full_target_path, os_slash_str(),);
        let metadata_string = match std::fs::read_to_string(&path_to_repo_metadata) {
            Ok(v) => v,
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!(
                        "Could not load copied metadata as string: {}",
                        e
                    )),
                )
            }
        };
        // - Make struct from metadata string
        let mut metadata_struct: BurritoMetadata = match serde_json::from_str(&metadata_string) {
            Ok(v) => v,
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Could not parse metadata: {}", e)),
                );
            }
        };

        // - Do the edits
        metadata_struct = match edit_metadata(state.app_resources_dir.clone(), metadata_struct, edits) {
            Ok(ms) => ms,
            Err(e) => return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Could not edit metadata: {}", e)),
                )
        };

        // - Write metadata
        let metadata_output_string = match serde_json::to_string(&metadata_struct) {
            Ok(s) => s,
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Could not make metadata as JSON: {}", e)),
                )
            }
        };
        match std::fs::write(path_to_repo_metadata, &metadata_output_string) {
            Ok(_) => (),
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Could not write metadata to repo: {}", e)),
                )
            }
        }
        // Maybe add gitignore file
        if add_ignore.is_some() && add_ignore.unwrap() {
            let path_to_gitignore_template = format!(
                "{}{}templates{}content_templates{}gitignore.txt",
                &state.app_resources_dir,
                os_slash_str(),
                os_slash_str(),
                os_slash_str(),
            );
            let gitignore_string = match std::fs::read_to_string(&path_to_gitignore_template) {
                Ok(v) => v,
                Err(e) => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!(
                            "Could not load gitignore template as string: {}",
                            e
                        )),
                    )
                }
            };
            let path_to_repo_gitignore =
                format!("{}{}.gitignore", full_target_path, os_slash_str(), );
            match std::fs::write(path_to_repo_gitignore, &gitignore_string) {
                Ok(_) => (),
                Err(e) => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!(
                            "Could not write gitignore to repo: {}",
                            e
                        )),
                    )
                }
            }
        }
        // Maybe delete src repo
        match delete_src {
            Some(true) => {
                match std::fs::remove_dir_all(full_src_path) {
                    Ok(_) => { },
                    Err(e) => return not_ok_json_response(
                        Status::BadRequest,
                        make_bad_json_data_response(format!("could not delete src repo: {}", e).to_string()),
                    ),
                }
            },
            _ => {}
        }
        ok_ok_json_response()
    } else {
        not_ok_bad_repo_json_response()
    }
}
