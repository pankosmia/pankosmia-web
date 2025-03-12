use std::fs;
use std::path::PathBuf;
use rocket::{get, post, State};
use rocket::response::{status};
use rocket::http::{ContentType, Status};
use serde_json::{Map, Value};
use std::collections::HashSet;
use rocket::serde::json::Json;
use crate::utils::paths::os_slash_str;
use crate::structs::{AppSettings};
use crate::utils::json_responses::{make_bad_json_data_response, make_good_json_data_response};

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
        return status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response(
                    format!("expected 0 - 2 filter terms, not {}", filter_items.len()).to_string(),
                ),
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
    match fs::read_to_string(path_to_serve) {
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
                    status::Custom(
                        Status::Ok,
                        (
                            ContentType::JSON,
                            serde_json::to_string(&negotiated).unwrap(),
                        ),
                    )
                }
                Err(e) => status::Custom(
                    Status::BadRequest,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response(
                            format!("could not parse for negotiated i18n: {}", e).to_string(),
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
                    format!("could not read for negotiated i18n: {}", e).to_string(),
                ),
            ),
        ),
    }
}

/// *`GET /flat/<filter>`*
///
/// Typically mounted as **`/i18n/flat/<filter>`**
///
/// Returns a flat object containing each i18n key with the best match based on the language preference settings. The optional filter restricts the keys returned. So, for `/i18n/flat/flavors` the response might be
///
/// ```text
/// {
///   "flavors:names:parascriptural/x-bcvArticles": "Articles by Verse",
///   "flavors:names:parascriptural/x-bcvImages": "Images by Verse",
///   "flavors:names:parascriptural/x-bcvNotes": "Notes by Verse",
///   "flavors:names:parascriptural/x-videolinks": "Video Links",
///   "flavors:names:scripture/textTranslation": "Scripture (Text)"
/// }
/// ```
#[get("/flat/<filter..>")]
pub async fn flat_i18n(
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
        return status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response(
                    format!("expected 0 - 2 filter terms, not {}", filter_items.len()).to_string(),
                ),
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
    match fs::read_to_string(path_to_serve) {
        Ok(v) => {
            match serde_json::from_str::<Value>(v.as_str()) {
                Ok(sj) => {
                    let languages = state.languages.lock().unwrap().clone();
                    let mut flat = Map::new();
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
                            for (i18n_term, term_languages) in terms.as_object().unwrap() {
                                'user_lang: for user_language in languages.clone() {
                                    for (i18n_language, translation) in
                                        term_languages.as_object().unwrap()
                                    {
                                        // println!("{} {}", i18n_language, languages[0]);
                                        if *i18n_language == user_language {
                                            let flat_key = format!(
                                                "{}:{}:{}",
                                                i18n_type.clone(),
                                                i18n_subtype.clone(),
                                                i18n_term.clone()
                                            );
                                            flat.insert(flat_key, translation.clone());
                                            break 'user_lang;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    status::Custom(
                        Status::Ok,
                        (ContentType::JSON, serde_json::to_string(&flat).unwrap()),
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
    match fs::read_to_string(path_to_serve) {
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
    match fs::read_to_string(path_to_serve) {
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

/// *`POST /`*
///
/// Typically mounted as **`/i18n`**
///
/// Replaces the local i18n file.
#[post(
    "/",
    format = "json",
    data = "<payload>"
)]
pub async fn post_i18n(
    payload: Json<Value>
) -> status::Custom<(ContentType, String)> {
    let serialized = payload.to_string();
    fs::write("/home/mark/Downloads/foo.json", serialized).unwrap();
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            make_good_json_data_response("ok".to_string()),
        ),
    )
}
