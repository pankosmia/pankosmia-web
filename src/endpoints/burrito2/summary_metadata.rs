use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};
use std::path::{Components, PathBuf};
use crate::structs::{AppSettings, MetadataSummary};
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_json_response,
};
use crate::utils::burrito::summary_metadata_from_file;

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
        let path_to_serve = format!(
            "{}{}{}{}metadata.json",
            state.repo_dir.lock().unwrap().clone(),
            os_slash_str(),
            &repo_path.display().to_string(),
            os_slash_str()
        );
        let summary = summary_metadata_from_file(path_to_serve).unwrap_or_else(|_| MetadataSummary {
            name: "? Bad Metadata JSON ?".to_string(),
            description: "?".to_string(),
            abbreviation: "?".to_string(),
            generated_date: "?".to_string(),
            flavor_type: "?".to_string(),
            flavor: "?".to_string(),
            language_code: "?".to_string(),
            script_direction: "?".to_string(),
            book_codes: vec![],
            timestamp: 0
        });
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
        not_ok_bad_repo_json_response()
    }
}
