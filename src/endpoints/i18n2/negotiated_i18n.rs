use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};
use serde_json::{Map, Value};
use std::path::PathBuf;

/// *`GET /negotiated/<filter>`*
///
/// Typically mounted as **`/i18n/negotiated/<filter>`**
///
/// Returns a nested object containing each i18n key with the best match based on the language preference settings. The optional filter restricts the keys returned. So, for `/i18n/negotiated/flavors` the response might be
///
/// ```text
/// {
///   "flavors": {
///     "names": {
///       "parascriptural/x-bcvArticles": {
///         "language": "en",
///         "translation": "Articles by Verse"
///       },
///       "parascriptural/x-bcvImages": {
///         "language": "en",
///         "translation": "Images by Verse"
///       },
///       "parascriptural/x-bcvNotes": {
///         "language": "en",
///         "translation": "Notes by Verse"
///       },
///       "parascriptural/x-videolinks": {
///         "language": "en",
///         "translation": "Video Links"
///       },
///       "scripture/textTranslation": {
///         "language": "en",
///         "translation": "Scripture (Text)"
///       }
///     }
///   }
/// }
/// ```
#[get("/negotiated/<filter..>")]
pub async fn negotiated_i18n(
    state: &State<AppSettings>,
    filter: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let path_to_serve = state.working_dir.clone() + os_slash_str() + "i18n.json";
    let filter_items: Vec<String> = filter
        .display()
        .to_string()
        .split('/')
        .map(String::from)
        .collect();
    if filter_items.len() > 2 {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(
                format!("expected 0 - 2 filter terms, not {}", filter_items.len()).to_string(),
            ),
        );
    }
    let mut type_filter: Option<String> = None;
    let mut subtype_filter: Option<String> = None;
    if filter_items.len() > 0 && filter_items[0] != "" {
        type_filter = Some(filter_items[0].clone());
        if filter_items.len() > 1 && filter_items[1] != "" {
            subtype_filter = Some(filter_items[1].clone());
        }
    }
    match std::fs::read_to_string(path_to_serve) {
        Ok(v) => {
            match serde_json::from_str::<Value>(v.as_str()) {
                Ok(sj) => {
                    let languages = state.languages.lock().unwrap().clone();
                    let mut negotiated = Map::new();
                    for (i18n_type, subtypes) in sj.as_object().unwrap() {
                        // println!("{}", i18n_type);
                        match type_filter.clone() {
                            Some(v) => {
                                if v != *i18n_type {
                                    continue;
                                }
                            }
                            None => {}
                        }
                        let mut negotiated_types = Map::new();
                        for (i18n_subtype, terms) in subtypes.as_object().unwrap() {
                            // println!("   {}", i18n_subtype);
                            match subtype_filter.clone() {
                                Some(v) => {
                                    if v != *i18n_subtype {
                                        continue;
                                    }
                                }
                                None => {}
                            }
                            let mut negotiated_terms = Map::new();
                            for (i18n_term, term_languages) in terms.as_object().unwrap() {
                                // println!("      {}", i18n_term);
                                let mut negotiated_translations = Map::new();
                                'user_lang: for user_language in languages.clone() {
                                    for (i18n_language, translation) in
                                        term_languages.as_object().unwrap()
                                    {
                                        // println!("{} {}", i18n_language, languages[0]);
                                        if *i18n_language == user_language {
                                            negotiated_translations.insert(
                                                "language".to_string(),
                                                Value::String(i18n_language.clone()),
                                            );
                                            negotiated_translations.insert(
                                                "translation".to_string(),
                                                translation.clone(),
                                            );
                                            break 'user_lang;
                                        }
                                    }
                                }
                                negotiated_terms.insert(
                                    i18n_term.clone(),
                                    Value::Object(negotiated_translations),
                                );
                            }
                            negotiated_types
                                .insert(i18n_subtype.clone(), Value::Object(negotiated_terms));
                        }
                        negotiated.insert(i18n_type.clone(), Value::Object(negotiated_types));
                    }
                    ok_json_response(serde_json::to_string(&negotiated).unwrap())
                }
                Err(e) => not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(
                        format!("could not parse for negotiated i18n: {}", e).to_string(),
                    ),
                ),
            }
        }
        Err(e) => not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(
                format!("could not read for negotiated i18n: {}", e).to_string(),
            ),
        ),
    }
}
