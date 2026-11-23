use crate::structs::{AppSettings, BurritoMetadata, EditMetadataRecord};
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_ok_json_response,
};
use crate::utils::burrito::edit_metadata;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, State};
use std::path::{Components, PathBuf};

/// *`POST /metadata/edit/<repo_path>`*
///
/// Typically mounted as **`/api/burrito/metadata/edit/<repo_path>`**
///
/// Edits the Dublin Core-like parts of the metadata for a repo.
/// The data is provided as a JSON payload:
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

#[post("/metadata/edit/<repo_path..>", format = "json", data = "<edits>")]
#[allow(irrefutable_let_patterns)]
pub async fn post_edit_metadata(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    edits: Json<EditMetadataRecord>,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    let full_repo_path = format!(
        "{}{}{}",
        state.repo_dir.lock().expect("repo dir lock"),
        os_slash_str(),
        &repo_path.display().to_string()
    );
    if check_path_components(&mut path_components.clone())
        && std::fs::metadata(&full_repo_path).is_ok()
    {
        // Get metadata as string
        let path_to_repo_metadata = format!("{}{}metadata.json", &full_repo_path, os_slash_str(),);
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
        // Make struct from metadata string
        let mut metadata_struct: BurritoMetadata = match serde_json::from_str(&metadata_string) {
            Ok(v) => v,
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Could not parse metadata: {}", e)),
                );
            }
        };

        // Do the edits
        metadata_struct = match edit_metadata(state.app_resources_dir.clone(), metadata_struct, edits) {
            Ok(ms) => ms,
            Err(e) => return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Could not edit metadata: {}", e)),
                )
        };

        // Write metadata
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
        ok_ok_json_response()
    } else {
        not_ok_bad_repo_json_response()
    }
}
