use crate::structs::{AppSettings};
use crate::utils::files::load_json;
use crate::utils::json_responses::{make_bad_json_data_response};
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_ok_json_response};
use crate::utils::time::utc_now_timestamp_string;
use git2::{Repository, RepositoryInitOptions};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, FromForm, State};
use rocket::serde::Deserialize;
use serde_json::{json, Value};

#[derive(FromForm, Deserialize)]
pub struct NewTextTranslationContentForm {
    pub content_name: String,
    pub content_abbr: String,
    pub content_type: String,
    pub content_language_code: String,
    pub content_language_name: Option<String>,
    pub add_book: bool,
    pub book_code: Option<String>,
    pub book_title: Option<String>,
    pub book_abbr: Option<String>,
    pub add_cv: Option<bool>,
    pub versification: Option<String>,
    pub branch_name: Option<String>
}

/// *`POST /new-text-translation`*
///
/// Typically mounted as **`/git/new-text-translation`**
///
/// Creates a new, local textTranslation repo. It requires the following fields as a JSON body:
/// - content_name (string)
/// - content_abbr (string)
/// - content_type (string)
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
        "{}{}templates{}content_templates{}{}{}metadata.json",
        &state.app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        json_form.content_type.clone(),
        os_slash_str(),
    );
    if !std::path::Path::new(&path_to_template).is_file() {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!(
                "Metadata template {} not found",
                json_form.content_type
            )),
        );
    }
    // Build path for new repo and parent
    let path_to_new_repo_parent = format!(
        "{}{}_local_{}_local_",
        state.repo_dir.lock().unwrap().clone(),
        os_slash_str(),
        os_slash_str(),
    );
    let path_to_new_repo = format!(
        "{}{}{}",
        path_to_new_repo_parent.clone(),
        os_slash_str(),
        json_form.content_abbr.clone()
    );
    // Check path doesn't already exist
    if std::path::Path::new(&path_to_new_repo).exists() {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!(
                "Local content called '{}' already exists",
                json_form.content_abbr
            )),
        );
    }
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
    // Init repo
    let final_new_branch_name = json_form.branch_name.clone().unwrap_or("master".to_string());
    let mut repo_options = RepositoryInitOptions::new();
    let repo_options2 = repo_options.initial_head(final_new_branch_name.as_str());
    let new_repo = match Repository::init_opts(&path_to_new_repo, &repo_options2) {
        Ok(repo) => repo,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not create repo: {}", e)),
            )
        }
    };
    // Set up local user info
    let mut config = new_repo.config().unwrap();
    config
        .set_str("user.name", whoami::username().as_str())
        .unwrap();
    config
        .set_str(
            "user.email",
            format!("{}@localhost", whoami::username().as_str()).as_str(),
        )
        .unwrap();
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

    // Copy gitignore file
    let path_to_gitignore_template = format!(
        "{}{}templates{}content_templates{}gitignore.txt",
        &state.app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
    );
    let gitignore_string = match std::fs::read_to_string(&path_to_gitignore_template) {
        Ok(v) => v,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not load gitignore template as string: {}",
                    e
                )),
            )
        }
    };
    let path_to_repo_gitignore =
        format!("{}{}.gitignore", path_to_new_repo, os_slash_str(),);
    match std::fs::write(path_to_repo_gitignore, &gitignore_string) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not write gitignore to repo: {}",
                    e
                )),
            )
        }
    }

    // Custom language begins with x- and name must be provided
    // Non-custom language must be in lookup, provided name is ignored
    let language_name;
    if json_form.content_language_code.starts_with("x-") {
        language_name = match json_form.content_language_name.clone() {
            Some(n) => n,
            None => return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!(
                    "Language code '{}' is custom ('x-') but no language name has been provided",
                    &json_form.content_language_code
                )),
            )
        }
    } else {
        // Read language lookup
        let path_to_language_lookup = format!(
            "{}{}app_resources{}lookups{}bcp47-language_codes.json",
            &state.app_resources_dir,
            os_slash_str(),
            os_slash_str(),
            os_slash_str(),
        );

        let language_lookup_json = match load_json(&path_to_language_lookup) {
            Ok(v) => v,
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!(
                        "Could not load and parse language lookup: {}",
                        e
                    )),
                )
            }
        };

        language_name = match language_lookup_json[&json_form.content_language_code].as_object() {
            Some(r) => r["en"].as_str().expect("English language name").to_string(),
            None => return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!(
                    "Language code '{}' is not custom (no 'x-') but has not been found in the BCP47 lookup table",
                    &json_form.content_language_code
                ))
            ),
        };
    }

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
        .replace("%%CREATED_TIMESTAMP%%", now_time.to_string().as_str())
        .replace(
            "%%LANGUAGE%%",
            serde_json::to_string(&language_json)
                .expect("language json")
                .as_str(),
        );
    // Get versification file as JSON
    let path_to_versification = format!(
        "{}{}templates{}content_templates{}vrs{}{}.json",
        &state.app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        json_form.versification.clone().unwrap(),
    );
    let versification_schema = match load_json(&path_to_versification) {
        Ok(j) => j,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not load versification JSON: {}", e)),
            )
        }
    };
    // Write it out to new repo
    let path_to_repo_versification =
        format!("{}{}ingredients/vrs.json", path_to_new_repo, os_slash_str(),);
    let versification_string = serde_json::to_string(&versification_schema).unwrap();
    match std::fs::write(path_to_repo_versification, &versification_string) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not write versification to repo: {}",
                    e
                )),
            )
        }
    }
    // Make new book if necessary:
    if json_form.add_book {
        let scope_string = format!("\"{}\": []", json_form.book_code.clone().unwrap().as_str());
        metadata_string = metadata_string.replace("%%SCOPE%%", scope_string.as_str());
        // - Read and customize USFM template
        let path_to_usfm_template = format!(
            "{}{}templates{}content_templates{}{}{}book.usfm",
            &state.app_resources_dir,
            os_slash_str(),
            os_slash_str(),
            os_slash_str(),
            json_form.content_type.clone(),
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
                            json_form.versification.clone().unwrap()
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
                            json_form.versification.clone().unwrap()
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
                            json_form.versification.clone().unwrap()
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
            usfm_string =
                usfm_string.replace("%%STUBCONTENT%%", "\\c 1\n\\p\n\\v 1\n___");
        }
        // - add ingredient to metadata
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
    // Add and commit
    new_repo
        .index()
        .unwrap()
        .add_all(&["."], git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    new_repo.index().unwrap().write().unwrap();
    let sig = new_repo.signature().unwrap();
    let tree_id = {
        let mut index = new_repo.index().unwrap();
        index.write_tree().unwrap()
    };
    let tree = new_repo.find_tree(tree_id).unwrap();
    new_repo
        .commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();
    ok_ok_json_response()
}
