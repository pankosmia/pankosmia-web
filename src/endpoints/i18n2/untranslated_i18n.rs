use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use serde_json::Value;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;

/// *`GET /untranslated/<lang>`*
///
/// Typically mounted as **`/i18n/untranslated/<lang>`**
///
/// Returns an array containing terms that are untranslated in the given language. So, for `/i18n/untranslated/de` the response might be
///
/// ```text
/// [
///   "components:framework:no_entry_if_offline",
///   "components:header:goto_local_projects_menu_item",
///   ...
/// ]
/// ````
#[get("/untranslated/<lang>")]
pub async fn untranslated_i18n(
    state: &State<AppSettings>,
    lang: String,
) -> status::Custom<(ContentType, String)> {
    let path_to_serve = state.working_dir.clone() + os_slash_str() + "i18n.json";
    match std::fs::read_to_string(path_to_serve) {
        Ok(v) => {
            match serde_json::from_str::<Value>(v.as_str()) {
                Ok(sj) => {
                    let mut untranslated: Vec<String> = Vec::new();
                    for (i18n_type, subtypes) in sj.as_object().unwrap() {
                        // println!("{}", i18n_type);
                        for (i18n_subtype, terms) in subtypes.as_object().unwrap() {
                            // println!("   {}", i18n_subtype);
                            for (i18n_term, term_languages) in terms.as_object().unwrap() {
                                // println!("      {}", i18n_term);
                                if !term_languages
                                    .as_object()
                                    .unwrap()
                                    .contains_key(lang.as_str())
                                {
                                    let flat_key = format!(
                                        "{}:{}:{}",
                                        i18n_type.clone(),
                                        i18n_subtype.clone(),
                                        i18n_term.clone()
                                    );
                                    untranslated.push(flat_key);
                                }
                            }
                        }
                    }
                    status::Custom(
                        Status::Ok,
                        (
                            ContentType::JSON,
                            serde_json::to_string(&untranslated).unwrap(),
                        ),
                    )
                }
                Err(e) => status::Custom(
                    Status::BadRequest,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response(
                            format!("could not parse for untranslated i18n: {}", e).to_string(),
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
                    format!("could not read for untranslated i18n: {}", e).to_string(),
                ),
            ),
        ),
    }
}