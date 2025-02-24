use std::fs;
use std::path::PathBuf;
use rocket::{get, State};
use rocket::response::{status};
use rocket::http::{ContentType, Status};
use serde_json::{Map, Value};
use std::collections::HashSet;
use crate::utils::paths::os_slash_str;
use crate::structs::AppSettings;
use crate::utils::json_responses::{make_bad_json_data_response};

#[get("/raw")]
pub(crate) async fn raw_i18n(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
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

#[get("/negotiated/<filter..>")]
pub(crate) async fn negotiated_i18n(
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

#[get("/flat/<filter..>")]
pub(crate) async fn flat_i18n(
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

#[get("/untranslated/<lang>")]
pub(crate) async fn untranslated_i18n(
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

#[get("/used-languages")]
pub(crate) async fn used_languages(
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
