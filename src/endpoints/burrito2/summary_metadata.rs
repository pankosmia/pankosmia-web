use crate::structs::{AppSettings, MetadataSummary};
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{not_ok_json_response, ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};
use serde_json::Value;
use std::path::{Components, PathBuf};

/// *`GET /metadata/summary/<repo_path>`*
///
/// Typically mounted as **`/burrito/metadata/summary/<repo_path>`**
///
/// Returns a flat summary of information from the raw metadata.json file for the specified burrito, where *repo_path* is *`<server>/<org>/<repo>`* and refers to a local repo. eg, the response to `/burrito/metadata/summary/git.door43.org/BurritoTruck/fr_psle` might be
///
/// ```text
/// {
///   "name": "Pain Sur Les Eaux",
///   "description": "Une traduction littéralement plus simple",
///   "abbreviation": "PSLE",
///   "generated_date": "2024-11-15T11:02:02.473Z",
///   "flavor_type": "scripture",
///   "flavor": "textTranslation",
///   "language_code": "fr",
///   "script_direction": "ltr"
/// }
/// ```
#[get("/metadata/summary/<repo_path..>")]
pub async fn summary_metadata(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let path_to_serve = state.repo_dir.lock().unwrap().clone()
            + os_slash_str()
            + &repo_path.display().to_string()
            + os_slash_str()
            + "metadata.json";
        let file_string = match std::fs::read_to_string(path_to_serve) {
            Ok(v) => v,
            Err(e) => {
                return not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(
                        format!("could not read metadata: {}", e).to_string(),
                    ),
                )
            }
        };
        let raw_metadata_struct: Value = match serde_json::from_str(file_string.as_str()) {
            Ok(v) => v,
            Err(e) => {
                return not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(
                        format!("could not parse metadata: {}", e).to_string(),
                    ),
                );
            }
        };
        let summary = MetadataSummary {
            name: raw_metadata_struct["identification"]["name"]["en"]
                .as_str()
                .unwrap()
                .to_string(),
            description: match raw_metadata_struct["identification"]["description"]["en"].clone() {
                Value::String(v) => v.as_str().to_string(),
                Value::Null => "".to_string(),
                _ => "?".to_string(),
            },
            abbreviation: match raw_metadata_struct["identification"]["abbreviation"]["en"].clone()
            {
                Value::String(v) => v.as_str().to_string(),
                Value::Null => "".to_string(),
                _ => "?".to_string(),
            },
            generated_date: match raw_metadata_struct["meta"]["dateCreated"].clone() {
                Value::String(v) => v.as_str().to_string(),
                Value::Null => "".to_string(),
                _ => "?".to_string(),
            },
            flavor_type: raw_metadata_struct["type"]["flavorType"]["name"]
                .as_str()
                .unwrap()
                .to_string(),
            flavor: raw_metadata_struct["type"]["flavorType"]["flavor"]["name"]
                .as_str()
                .unwrap()
                .to_string(),
            language_code: raw_metadata_struct["languages"][0]["tag"]
                .as_str()
                .unwrap()
                .to_string(),
            script_direction: match raw_metadata_struct["languages"][0]["scriptDirection"].clone() {
                Value::String(v) => v.as_str().to_string(),
                _ => "?".to_string(),
            },
        };
        match serde_json::to_string(&summary) {
            Ok(v) => ok_json_response(v),
            Err(e) => not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(
                    format!("could not serialize metadata: {}", e).to_string(),
                ),
            ),
        }
    } else {
        not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response("bad repo path!".to_string()),
        )
    }
}
