use crate::structs::{
    AppSettings, BurritoMetadata, BurritoMetadataIngredient, NewScriptureBookForm,
};
use crate::utils::files::load_json;
use crate::utils::json_responses::{make_bad_json_data_response};
use crate::utils::paths::{check_local_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_ok_json_response,
};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, State};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Components, PathBuf};

/// *`POST /new-scripture-book/<repo_path>`*
///
/// Typically mounted as **`/git/new-scripture-book/<repo_path>`**
///
/// Adds a Scripture book to a local repo at the given repo path.
///
///  It requires the following fields as a JSON body:
/// - book_code (string)
/// - book_title (string)
/// - book_abbr (string)
/// - add_cv (boolean)
#[allow(irrefutable_let_patterns)]
#[post(
    "/new-scripture-book/<repo_path..>",
    format = "json",
    data = "<json_form>"
)]
pub async fn new_scripture_book(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    json_form: Json<NewScriptureBookForm>,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_local_path_components(&mut path_components.clone()) {
        // Read metadata
        let repo_dir_path = state.repo_dir.lock().unwrap().clone();
        let repo_name = path_components
            .skip(2)
            .next()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap();
        let path_to_repo_metadata = format!(
            "{}{}_local_{}_local_{}{}{}metadata.json",
            repo_dir_path,
            os_slash_str(),
            os_slash_str(),
            os_slash_str(),
            &repo_name,
            os_slash_str(),
        );
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
        // Make struct from metadata
        let metadata_struct: BurritoMetadata = match serde_json::from_str(&metadata_string) {
            Ok(v) => v,
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Could not parse metadata: {}", e)),
                );
            }
        };
        // Check new book isn't already there
        let new_ingredients_path = format!("ingredients/{}.usfm", &json_form.book_code);
        if let ingredients = metadata_struct.ingredients.lock().unwrap() {
            if ingredients.contains_key(&new_ingredients_path) {
                return not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(format!(
                        "Book '{}' already exists in metadata",
                        &json_form.book_code
                    )),
                );
            }
        }
        // Make USFM with optional cv
        let path_to_usfm_template = format!(
            "{}{}templates{}content_templates{}text_translation{}book.usfm",
            &state.app_resources_dir,
            os_slash_str(),
            os_slash_str(),
            os_slash_str(),
            os_slash_str(),
        );
        let mut usfm_string = match std::fs::read_to_string(&path_to_usfm_template) {
            Ok(v) => v,
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!(
                        "Could not load USFM template as string: {}",
                        e
                    )),
                )
            }
        };
        usfm_string = usfm_string
            .replace("%%BOOKCODE%%", json_form.book_code.clone().as_str())
            .replace("%%BOOKNAME%%", json_form.book_title.clone().as_str())
            .replace("%%CONTENTNAME%%", &repo_name)
            .replace("%%BOOKABBR%%", json_form.book_abbr.clone().as_str());
        // - If ve
        if json_form.add_cv {
            // Load vrs file from repo
            let path_to_repo_vrs = format!(
                "{}{}_local_{}_local_{}{}{}ingredients{}vrs.json",
                repo_dir_path,
                os_slash_str(),
                os_slash_str(),
                os_slash_str(),
                &repo_name,
                os_slash_str(),
                os_slash_str(),
            );
            let versification_ob = match load_json(&path_to_repo_vrs) {
                Ok(j) => j,
                Err(e) => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!(
                            "Could not load repo versification JSON: {}",
                            e
                        )),
                    )
                }
            };
            // Generate cv USFM
            let mut cv_bits = Vec::new();
            let max_verses_ob = match &versification_ob["maxVerses"] {
                Value::Object(o) => o,
                _ => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(
                            "Could not find maxVerses in versification JSON for this repo"
                                .to_string(),
                        ),
                    )
                }
            };
            let book_max_verses_arr =
                match &max_verses_ob[&json_form.book_code.clone()] {
                    Value::Array(a) => a,
                    _ => return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!(
                            "Could not find maxVerses for {} in versification JSON for this repo",
                            json_form.book_code.clone(),
                        )),
                    ),
                };
            let mut chapter_number = 0;
            for max_verse in book_max_verses_arr {
                chapter_number += 1;
                cv_bits.push(format!("\\c {}", chapter_number));
                cv_bits.push("\\p".to_string());
                let max_verse_number = max_verse.as_str().unwrap().parse::<i32>().unwrap();
                for verse_number in 1..=max_verse_number {
                    cv_bits.push(format!("\\v {}", verse_number));
                }
            }
            // Insert
            usfm_string = usfm_string.replace("%%STUBCONTENT%%", cv_bits.join("\n").as_str());
        } else {
            usfm_string =
                usfm_string.replace("%%STUBCONTENT%%", "\\c 1\n\\p\n\\v 1\nFirst verse...");
        }
        // Save USFM
        let path_to_new_book = format!(
            "{}{}_local_{}_local_{}{}{}ingredients{}{}.usfm",
            repo_dir_path,
            os_slash_str(),
            os_slash_str(),
            os_slash_str(),
            &repo_name,
            os_slash_str(),
            os_slash_str(),
            &json_form.book_code
        );
        match std::fs::write(path_to_new_book, &usfm_string) {
            Ok(_) => (),
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Could not write usfm to repo: {}", e)),
                )
            }
        }
        // Add ingredient record for USFM
        if let mut ingredients = metadata_struct.ingredients.lock().unwrap() {
            let mut new_ingredients = BTreeMap::new();
            for (k, v) in ingredients.iter() {
                new_ingredients.insert(k.clone(), v.clone());
            }
            let ingredient_key = format!("ingredients/{}.usfm", &json_form.book_code);
            let ingredient_struct = BurritoMetadataIngredient {
                checksum: json!({"md5": format!("{:?}", md5::compute(&usfm_string))}),
                mimeType: "text/plain".to_string(),
                scope: Some(json!({json_form.book_code.to_string(): []})),
                size: usfm_string.len(),
            };
            new_ingredients.insert(ingredient_key, ingredient_struct);
            *ingredients = new_ingredients;
        }
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
        // Add and commit
        ok_ok_json_response()
    } else {
        not_ok_bad_repo_json_response()
    }
}
