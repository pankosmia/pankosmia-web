use crate::structs::AppSettings;
use crate::utils::burrito::{copy_gitignore_template, copy_vrs_template, language_name_from_code};
use crate::utils::files::paths_to_new_burrito;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_ok_json_response};
use crate::utils::time::utc_now_timestamp_string;
use crate::utils::git::{init_repo, add_and_commit_repo};
use rocket::http::{Status, ContentType};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{post, FromForm, State};
use serde_json::{json, Value};

#[derive(FromForm, Deserialize)]
pub struct NewTextTranslationContentForm {
    pub content_name: String,
    pub content_abbr: String,
    pub copyright: Option<String>,
    pub content_language_code: String,
    pub content_language_name: Option<String>,
    pub add_book: bool,
    pub book_code: Option<String>,
    pub book_title: Option<String>,
    pub book_abbr: Option<String>,
    pub add_cv: Option<bool>,
    pub versification: String,
    pub branch_name: Option<String>,
}

/// *`POST /new-text-translation`*
///
/// Typically mounted as **`/git/new-text-translation`**
///
/// Creates a new, local textTranslation repo. It requires the following fields as a JSON body:
/// - content_name (string)
/// - content_abbr (string)
/// - copyright (null or string)
/// - content_language_code
/// - content_language_name (optional)
/// - versification (string)
/// - add_book (boolean)
/// - book_code (null or string)
/// - book_title (null or string)
/// - book_abbr (null or string)
/// - add_cv (null or boolean)
/// - branch_name (null or string)
#[post("/new-text-translation", format = "json", data = "<json_form>")]
pub fn new_text_translation_repo(
    state: &State<AppSettings>,
    json_form: Json<NewTextTranslationContentForm>,
) -> status::Custom<(ContentType, String)> {
    // Check template type exists
    let path_to_template = format!(
        "{}{}templates{}content_templates{}text_translation{}metadata.json",
        &state.app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
    );
    if !std::path::Path::new(&path_to_template).is_file() {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(
                "Metadata template for text_translation not found".to_string(),
            ),
        );
    }

    let language_name = match language_name_from_code(
        &state.app_resources_dir,
        json_form.content_language_code.clone(),
        json_form.content_language_name.clone(),
    ) {
        Ok(v) => v,
        Err(e) => {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!(
                    "Unable to find language name: {}: {}",
                    &json_form.content_language_code, e
                )),
            );
        }
    };

    let (path_to_new_repo_parent, path_to_new_repo) =
        match paths_to_new_burrito(&state.repo_dir.lock().unwrap(), &json_form.content_abbr) {
            Ok(tup) => tup,
            Err(e) => {
                return not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(format!("Could not get new repo paths: {}", e)),
                )
            }
        };

    // Make parents?
    match std::fs::create_dir_all(path_to_new_repo_parent) {
        Ok(_) => (),
        Err(e) => {
            return status::Custom(
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!(
                        "Could not create local content directories: {}",
                        e
                    )),
                ),
            )
        }
    }
    let new_repo = match init_repo(&path_to_new_repo, json_form.branch_name.clone()) {
        Ok(r) => r,
        Err(e) => {
            return status::Custom(
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!(
                        "{}",
                        e
                    )),
                ),
            )
        }
    };
    // Make ingredients dir
    let path_to_ingredients = format!("{}{}ingredients", path_to_new_repo, os_slash_str(),);
    match std::fs::create_dir(&path_to_ingredients) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not create ingredients directory for repo: {}",
                    e
                )),
            )
        }
    }

    match copy_gitignore_template(&state.app_resources_dir, &path_to_new_repo) {
        Ok(_) => {}
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("{}", e)),
            )
        }
    };

    // Read and customize metadata
    let mut metadata_string = match std::fs::read_to_string(&path_to_template) {
        Ok(v) => v,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not load metadata template as string: {}",
                    e
                )),
            )
        }
    };
    let now_time = utc_now_timestamp_string();
    let language_json = json!(
        {
            "tag": &json_form.content_language_code,
            "name": {
                "en": &language_name,
        }
        }
    );
    metadata_string = metadata_string
        .replace("%%ABBR%%", json_form.content_abbr.as_str())
        .replace("%%CONTENT_NAME%%", json_form.content_name.as_str())
        .replace(
            "%%COPYRIGHT%%",
            json_form
                .copyright
                .clone()
                .unwrap_or("unspecified".to_string())
                .as_str(),
        )
        .replace("%%CREATED_TIMESTAMP%%", now_time.to_string().as_str())
        .replace(
            "%%LANGUAGE%%",
            serde_json::to_string(&language_json)
                .expect("language json")
                .as_str(),
        );

    let versification_schema = match copy_vrs_template(
        &state.app_resources_dir,
        &path_to_new_repo,
        &json_form.versification,
    ) {
        Ok(v) => v,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("{}", e)),
            )
        }
    };

    // Make new book if necessary:
    if json_form.add_book {
        let scope_string = format!("\"{}\": []", json_form.book_code.clone().unwrap().as_str());
        metadata_string = metadata_string.replace("%%SCOPE%%", scope_string.as_str());
        // - Read and customize USFM template
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
            .replace(
                "%%BOOKCODE%%",
                json_form.book_code.clone().unwrap().as_str(),
            )
            .replace(
                "%%BOOKNAME%%",
                json_form.book_title.clone().unwrap().as_str(),
            )
            .replace("%%CONTENTNAME%%", json_form.content_name.clone().as_str())
            .replace(
                "%%BOOKABBR%%",
                json_form.book_abbr.clone().unwrap().as_str(),
            );
        // - If ve
        if json_form.add_cv.unwrap() {
            // Generate cv USFM
            let mut cv_bits = Vec::new();
            let versification_ob = match &versification_schema {
                Value::Object(o) => o,
                _ => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!(
                            "Could not find versification JSON object for {}",
                            json_form.versification.clone()
                        )),
                    )
                }
            };
            let max_verses_ob = match &versification_ob["maxVerses"] {
                Value::Object(o) => o,
                _ => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!(
                            "Could not find maxVerses in versification JSON for {}",
                            json_form.versification.clone()
                        )),
                    )
                }
            };
            let book_max_verses_arr = match &max_verses_ob[&json_form.book_code.clone().unwrap()] {
                Value::Array(a) => a,
                _ => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!(
                            "Could not find maxVerses for {} in versification JSON for {}",
                            json_form.book_code.clone().unwrap(),
                            json_form.versification.clone()
                        )),
                    )
                }
            };
            let mut chapter_number = 0;
            for max_verse in book_max_verses_arr {
                chapter_number += 1;
                cv_bits.push(format!("\\c {}", chapter_number));
                cv_bits.push("\\p".to_string());
                let max_verse_number = max_verse.as_str().unwrap().parse::<i32>().unwrap();
                for verse_number in 1..=max_verse_number {
                    cv_bits.push(format!("\\v {} ___", verse_number));
                }
            }
            // Insert
            usfm_string = usfm_string.replace("%%STUBCONTENT%%", cv_bits.join("\n").as_str());
        } else {
            usfm_string = usfm_string.replace("%%STUBCONTENT%%", "\\c 1\n\\p\n\\v 1\n___");
        }
        // - add ingredient to metadata
        let versification_string = serde_json::to_string(&versification_schema).unwrap();
        let ingredient_json = json!(
            {
                format!("ingredients/{}.usfm", json_form.book_code.clone().unwrap()): {
                    "checksum": {
                        "md5": format!("{:?}", md5::compute(&usfm_string))
                    },
                    "mimeType": "text/plain",
                    "size": usfm_string.len(),
                    "scope": {
                        format!("{}", json_form.book_code.clone().unwrap()): []
                    }
                },
                "ingredients/vrs.json": {
                    "checksum": {
                        "md5": format!("{:?}", md5::compute(&versification_string))
                    },
                    "mimeType": "application/json",
                    "size": versification_string.len()
                }
            }
        );
        metadata_string = metadata_string.replace(
            "\"ingredients\": {}",
            format!("\"ingredients\": {}", ingredient_json.to_string().as_str()).as_str(),
        );
        // - Write USFM
        let path_to_usfm_destination = format!(
            "{}{}ingredients{}{}.usfm",
            &path_to_new_repo,
            os_slash_str(),
            os_slash_str(),
            json_form.book_code.clone().unwrap(),
        );
        match std::fs::write(path_to_usfm_destination, usfm_string) {
            Ok(_) => (),
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Could not write usfm to repo: {}", e)),
                )
            }
        }
    } else {
        // No ingredients
        metadata_string = metadata_string.replace("%%SCOPE%%", "");
    }
    // Write metadata
    let path_to_repo_metadata = format!("{}{}metadata.json", &path_to_new_repo, os_slash_str());
    match std::fs::write(path_to_repo_metadata, metadata_string) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not write metadata template to repo: {}",
                    e
                )),
            )
        }
    }
    match add_and_commit_repo(new_repo, &"Initial Commit".to_string()) {
        Ok(_) => {},
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "{}",
                    e
                )),
            )
        }
    };
    ok_ok_json_response()
}
