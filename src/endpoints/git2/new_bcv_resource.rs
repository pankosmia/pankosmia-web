use crate::structs::{AppSettings, NewBcvResourceContentForm};
use crate::utils::files::load_json;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_ok_json_response};
use chrono::Utc;
use git2::Repository;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, State};
use serde_json::json;

/// *`POST /new-bcv-resource`*
///
/// Typically mounted as **`/git/new-bcv-resource`**
///
/// Creates a new, local x-bcv* repo. It requires the following fields as a JSON body:
/// - content_name (string)
/// - content_abbr (string)
/// - tsv_type (string)
/// - content_language_code
/// - versification (string)
/// - add_book (boolean)
/// - book_code (null or string)
/// - book_title (null or string)
/// - book_abbr (null or string)
#[post("/new-bcv-resource", format = "json", data = "<json_form>")]
pub fn new_bcv_resource_repo(
    state: &State<AppSettings>,
    json_form: Json<NewBcvResourceContentForm>,
) -> status::Custom<(ContentType, String)> {
    // Read tsv catalog
    let path_to_tsv_catalog = format!(
        "{}{}app_resources{}tsv{}templates.json",
        &state.app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
    );
    // Load tsv catalog
    let tsv_catalog = match load_json(&path_to_tsv_catalog) {
        Ok(j) => j,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not load tsv template JSON: {}", e)),
            )
        }
    };
    // Get flavor from tsv type
    let tsv_type_record = match tsv_catalog[json_form.tsv_type.clone()].as_object() {
        Some(o) => o,
        None => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not find record for {} in tsv catalog JSON",
                    json_form.tsv_type
                )),
            )
        }
    };
    let tsv_type_flavor = match tsv_type_record["flavor"].as_str() {
        Some(v) => v,
        None => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not find flavor in record for {} in tsv catalog JSON",
                    json_form.tsv_type
                )),
            )
        }
    };
    // Check template type exists
    let path_to_template = format!(
        "{}{}templates{}content_templates{}{}{}metadata.json",
        &state.app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        tsv_type_flavor,
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
    let new_repo = match Repository::init(&path_to_new_repo) {
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
    let now_time = Utc::now();
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
}
