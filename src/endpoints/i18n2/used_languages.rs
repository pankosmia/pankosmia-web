use std::collections::HashSet;
use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use serde_json::Value;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;

/// *`GET /used-languages`*
///
/// Typically mounted as **`/i18n/used-languages`**
///
/// Returns an array containing languages into which at least one term is translated.
///
/// `["en","fr"]`
#[get("/used-languages")]
pub async fn used_languages(
    state: &State<AppSettings>
) -> status::Custom<(ContentType, String)> {
    let path_to_serve = state.working_dir.clone() + os_slash_str() + "i18n.json";
    match std::fs::read_to_string(path_to_serve) {
        Ok(v) => {
            match serde_json::from_str::<Value>(v.as_str()) {
                Ok(sj) => {
                    let mut used = HashSet::new();
                    for (_, subtypes) in sj.as_object().unwrap() {
                        for (_, terms) in subtypes.as_object().unwrap() {
                            for (_, term_languages) in terms.as_object().unwrap() {
                                for (i18n_language, _) in term_languages.as_object().unwrap()
                                {
                                    used.insert(i18n_language.clone());
                                }
                            }
                        }
                    }
                    status::Custom(
                        Status::Ok,
                        (ContentType::JSON, serde_json::to_string(&used).unwrap()),
                    )
                }
                Err(e) => status::Custom(
                    Status::BadRequest,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response(
                            format!("could not parse for flat i18n: {}", e).to_string(),
                        ),
                    ),
                ),
            }
        }
        Err(e) => status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response(
                    format!("could not read for flat i18n: {}", e).to_string(),
                ),
            ),
        ),
    }
}