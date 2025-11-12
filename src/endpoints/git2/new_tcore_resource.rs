use std::fs;
use crate::structs::{AppSettings, BurritoMetadata};
use crate::utils::files::load_json;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_json_response, ok_ok_json_response};
use git2::{Repository, RepositoryInitOptions};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, FromForm, State};
use rocket::serde::{Deserialize, Serialize};
use serde_json::json;
use crate::utils::time::utc_now_timestamp_string;

/// *`POST /new-tcore-resource`*
///
/// Typically mounted as **`/git/new-tcore-resource`**
///
/// Creates a new, local x-tcore* repo. It requires the following fields as a JSON body:
/// - usfm_repo_path (string)
/// - book_code (string)

#[derive(FromForm, Deserialize, Serialize, Debug, Clone)]
pub struct NewBcvResourceContentForm {
    pub usfm_repo_path: String,
    pub book_code: String
}

#[post("/new-tcore-resource", format = "json", data = "<json_form>")]
pub fn new_tcore_resource_repo(
    state: &State<AppSettings>,
    json_form: Json<NewBcvResourceContentForm>,
) -> status::Custom<(ContentType, String)> {
    // Check template exists
    let path_to_template = format!(
        "{}{}templates{}content_templates{}x-tcore{}metadata.json",
        &state.app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
    );
    if !std::path::Path::new(&path_to_template).is_file() {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!(
                "Metadata template {} not found",
                path_to_template
            )),
        );
    }
    let path_to_repo_dir = format!(
        "{}",
        state.repo_dir.lock().unwrap().clone()
    );
    let path_to_usfm_source_repo = format!(
        "{}{}{}",
        &path_to_repo_dir,
        os_slash_str(),
        &json_form.usfm_repo_path.replace("/", os_slash_str())
    );
    let path_to_usfm_source_repo_metadata = format!(
        "{}{}metadata.json",
        &path_to_usfm_source_repo,
        os_slash_str()
    );
    // Read the source repo metadata as JSON
    let source_metadata_string = match fs::read_to_string(&path_to_usfm_source_repo_metadata) {
        Ok(s) => s,
        Err(e) => return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!(
                "Could not read source metadata: '{}'",
                e
            )),
        )
    };
    let source_metadata: BurritoMetadata = match serde_json::from_str(&source_metadata_string) {
        Ok(j) => j,
        Err(_e) => return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!(
                "Could not read source metadata as JSON: '{}'",
                path_to_usfm_source_repo_metadata
            )),
        )
    };
    let source_identification = source_metadata.identification;
    let source_abbr_object = source_identification["abbreviation"].as_object().expect("identification abbreviation object");
    let source_abbr = source_abbr_object["en"].clone();
    println!("\n\n\n\n\n#### ABBR {} ####\n", source_abbr);
    let source_language = source_metadata.languages[0].as_object().unwrap();
    let source_language_tag = source_language["tag"].clone();
    println!("#### LANG {} ####\n\n\n\n\n", source_language_tag);
    ok_ok_json_response()
/*
    // TODO assemble repo name
    let path_to_usfm_source_repo_usfm = format!(
        "{}{}{}.usfm",
        &path_to_usfm_source_repo,
        os_slash_str(),
        json_form.book_code
    );

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
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not create local content directories: {}",
                    e
                )),
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
    metadata_string = metadata_string
        .replace("%%ABBR%%", json_form.content_abbr.as_str())
        .replace("%%CONTENT_NAME%%", json_form.content_name.as_str())
        .replace("%%CONTENT_NAME%%", json_form.content_name.as_str())
        .replace("%%CREATED_TIMESTAMP%%", now_time.to_string().as_str());
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
        // - Read TSV template
        let path_to_tsv_template = format!(
            "{}{}app_resources{}tsv{}{}.tsv",
            &state.app_resources_dir,
            os_slash_str(),
            os_slash_str(),
            os_slash_str(),
            json_form.tsv_type
        );
        let tsv_string = match std::fs::read_to_string(&path_to_tsv_template) {
            Ok(v) => v,
            Err(e) => {
                return status::Custom(
                    Status::InternalServerError,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response(format!(
                            "Could not load TSV template {} as string: {}",
                            json_form.tsv_type, e
                        )),
                    ),
                )
            }
        };
        // - add ingredient to metadata
        let ingredient_json = json!(
            {
                format!("ingredients/{}.tsv", json_form.book_code.clone().unwrap()): {
                    "checksum": {
                        "md5": format!("{:?}", md5::compute(&tsv_string))
                    },
                    "mimeType": "text/tab-separated-values",
                    "size": tsv_string.len(),
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
        // - Write TSV
        let path_to_tsv_destination = format!(
            "{}{}ingredients{}{}.tsv",
            &path_to_new_repo,
            os_slash_str(),
            os_slash_str(),
            json_form.book_code.clone().unwrap(),
        );
        match std::fs::write(path_to_tsv_destination, tsv_string) {
            Ok(_) => (),
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Could not write tsv to repo: {}", e)),
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
 */
}
