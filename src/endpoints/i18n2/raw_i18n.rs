use std::fs;
use rocket::{get, State};
use rocket::response::{status};
use rocket::http::{ContentType, Status};
use crate::utils::paths::os_slash_str;
use crate::structs::{AppSettings};
use crate::utils::json_responses::{make_bad_json_data_response};

/// *```GET /raw```*
///
/// Typically mounted as **`/i18n/raw/`**
///
/// Returns the raw, nested i18n.json file from the server.
///
/// ```text
/// {
///   "branding": {
///
///   },
///   "components": {
///     "framework": {
///       "no_entry_if_offline": {
///         "en": "You need to be online to view this page.",
///         "fr": "Vous devez vous connecter à l'Internet pour accéder à cette page."
///       }
///     },
///     "header": {
///       "goto_local_projects_menu_item": {
///         "en": "Projects on this machine",
///         "fr": "Projets sur cette machine"
///       },
///       "new_reference": {
///         "en": "New Reference",
///         "fr": "Nouvelle référence"
///       }
///     }
///   },
///   "flavors": {
///   ...
/// }
/// ```
#[get("/raw")]
pub async fn raw_i18n(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let path_to_serve = state.working_dir.clone() + os_slash_str() + "i18n.json";
    match fs::read_to_string(path_to_serve) {
        Ok(v) => status::Custom(Status::Ok, (ContentType::JSON, v)),
        Err(e) => status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response(format!("could not read raw i18n: {}", e).to_string()),
            ),
        ),
    }
}