use crate::structs::{AppSettings, BurritoMetadata};
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_ok_json_response};
use crate::utils::time::utc_now_timestamp_string;
use git2::{Repository, RepositoryInitOptions};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::{post, FromForm, State};
use std::fs;

/// *`POST /new-tcore-resource`*
///
/// Typically mounted as **`/git/new-tcore-resource`**
///
/// Creates a new, local x-tcore* repo. It requires the following fields as a JSON body:
/// - usfm_repo_path (string)
/// - book_code (string)
/// - branch_name (Optional string)

#[derive(FromForm, Deserialize, Serialize, Debug, Clone)]
pub struct NewBcvResourceContentForm {
    pub usfm_repo_path: String,
    pub book_code: String,
    pub branch_name: Option<String>
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
    let path_to_repo_dir = format!("{}", state.repo_dir.lock().unwrap().clone());
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
        Err(e) => {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!("Could not read source metadata: '{}'", e)),
            )
        }
    };
    let source_metadata: BurritoMetadata = match serde_json::from_str(&source_metadata_string) {
        Ok(j) => j,
        Err(_e) => {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!(
                    "Could not read source metadata as JSON: '{}'",
                    path_to_usfm_source_repo_metadata
                )),
            )
        }
    };
    // Extract useful info from source burrito
    let source_identification = source_metadata.identification;
    let source_abbr_object = source_identification["abbreviation"]
        .as_object()
        .expect("identification abbreviation object");
    let source_abbr_string = source_abbr_object["en"].clone();
    let source_abbr = source_abbr_string.as_str().unwrap();
    let source_name_object = source_identification["name"]
        .as_object()
        .expect("identification name object");
    let source_name_string = source_name_object["en"].clone();
    let source_name = source_name_string.as_str().unwrap();
    let source_language = source_metadata.languages[0].as_object().unwrap();
    let source_language_tag_string = source_language["tag"].clone();
    let source_language = source_language_tag_string.as_str().unwrap();
    let repo_name = format!("{}_tcchecks", &source_abbr);

    // Build path for new repo and parent
    let path_to_new_repo = format!(
        "{}{}_local_{}_local_{}{}",
        &path_to_repo_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        &repo_name
    );
    // Check path doesn't already exist
    if std::path::Path::new(&path_to_new_repo).exists() {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!(
                "Local content called '{}' already exists",
                &repo_name
            )),
        );
    }
    // Make parents?
    match fs::create_dir_all(path_to_repo_dir) {
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
    let final_new_branch_name = json_form
        .branch_name
        .clone()
        .unwrap_or("master".to_string());
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

    // Copy gitignore file
    let path_to_gitignore_template = format!(
        "{}{}templates{}content_templates{}gitignore.txt",
        &state.app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
    );
    let gitignore_string = match fs::read_to_string(&path_to_gitignore_template) {
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
    let path_to_repo_gitignore = format!("{}{}.gitignore", path_to_new_repo, os_slash_str(),);
    match fs::write(path_to_repo_gitignore, &gitignore_string) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not write gitignore to repo: {}", e)),
            )
        }
    }

    // Make bookProjects dir
    let path_to_book_projects = format!(
        "{}{}ingredients{}bookProjects{}{}",
        path_to_new_repo,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        format!(
            "{}_{}_{}_book",
            &source_language,
            &source_abbr,
            &json_form.book_code
        )
    );
    match fs::create_dir_all(&path_to_book_projects) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not create bookProjects directories for repo: {}",
                    e
                )),
            )
        }
    }

    // Copy chosen USFM file
    let path_to_usfm_source_repo_usfm = format!(
        "{}{}{}.usfm",
        &path_to_usfm_source_repo,
        os_slash_str(),
        json_form.book_code
    );
    let path_to_target_usfm = format!(
        "{}{}{}.usfm",
        &path_to_book_projects,
        os_slash_str(),
        json_form.book_code
    );
    let usfm_string = match fs::read_to_string(&path_to_usfm_source_repo_usfm) {
        Ok(v) => v,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not read usfm from source repo: {}",
                    e
                )),
            )
        }
    };
    match fs::write(path_to_target_usfm, usfm_string) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not write usfm to repo: {}",
                    e
                )),
            )
        }
    }

    // Read and customize metadata
    let mut metadata_string = match fs::read_to_string(&path_to_template) {
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
        .replace("%%ABBR%%", repo_name.as_str())
        .replace("%%CONTENT_NAME%%", format!("tC Checks ({})", source_name).as_str())
        .replace("%%CREATED_TIMESTAMP%%", now_time.to_string().as_str());
    // No ingredients
    metadata_string = metadata_string.replace("%%SCOPE%%", "");
    // Write metadata
    let path_to_repo_metadata = format!("{}{}metadata.json", &path_to_new_repo, os_slash_str());
    match fs::write(path_to_repo_metadata, metadata_string) {
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
